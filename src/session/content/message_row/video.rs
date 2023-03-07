use glib::clone;
use gst::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};
use tdlib::enums::MessageContent;

use crate::session::content::message_row::{
    MediaPicture, MessageBase, MessageBaseImpl, MessageBubble,
};
use crate::tdlib::Message;
use crate::utils::{parse_formatted_text, spawn};
use crate::Session;

use super::base::MessageBaseExt;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    template MessageVideo : .MessageBase {
        .MessageBubble message_bubble {
            styles ["media"]

            prefix: Overlay {
                .MessageMediaPicture picture {}

                [overlay]
                Label indicator {
                    halign: start;
                    valign: start;

                    styles ["osd-indicator"]
                }
            };
        }
    }
    "#)]
    pub(crate) struct MessageVideo {
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) message: RefCell<Option<Message>>,
        pub(super) is_animation: Cell<bool>,
        pub(super) pipeline: OnceCell<gst::Pipeline>,
        pub(super) file_src: OnceCell<gst::Element>,
        #[template_child]
        pub(super) message_bubble: TemplateChild<MessageBubble>,
        #[template_child]
        pub(super) picture: TemplateChild<MediaPicture>,
        #[template_child]
        pub(super) indicator: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageVideo {
        const NAME: &'static str = "MessageVideo";
        type Type = super::MessageVideo;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageVideo {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<Message>("message")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "message" => obj.set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => self.message.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().create_pipeline();
        }

        fn dispose(&self) {
            if let Some(pipeline) = self.pipeline.get() {
                pipeline
                    .set_state(gst::State::Null)
                    .expect("Unable to set the pipeline to the `Null` state");
                pipeline.bus().unwrap().remove_watch().unwrap();
            }
        }
    }

    impl WidgetImpl for MessageVideo {
        fn map(&self) {
            self.parent_map();

            self.pipeline
                .get()
                .unwrap()
                .set_state(gst::State::Playing)
                .unwrap();
        }

        fn unmap(&self) {
            self.parent_unmap();

            self.pipeline
                .get()
                .unwrap()
                .set_state(gst::State::Paused)
                .unwrap();
        }
    }

    impl MessageBaseImpl for MessageVideo {}
}

glib::wrapper! {
    pub(crate) struct MessageVideo(ObjectSubclass<imp::MessageVideo>)
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessageVideo {
    type Message = Message;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        if let Some(old_message) = imp.message.take() {
            let handler_id = imp.handler_id.take().unwrap();
            old_message.disconnect(handler_id);
        }

        imp.message_bubble.update_from_message(&message, true);

        let handler_id =
            message.connect_content_notify(clone!(@weak self as obj => move |message, _| {
                obj.update_content(message.content().0, &message.chat().session());
            }));
        imp.handler_id.replace(Some(handler_id));

        self.update_content(message.content().0, &message.chat().session());

        imp.message.replace(Some(message));
        self.notify("message");
    }
}

impl MessageVideo {
    fn create_pipeline(&self) {
        let imp = self.imp();
        let pipeline = gst::Pipeline::new(None);

        let src = gst::ElementFactory::make("filesrc").build().unwrap();
        let decodebin = gst::ElementFactory::make("decodebin").build().unwrap();
        let gtksink = gst::ElementFactory::make("gtk4paintablesink")
            .build()
            .unwrap();

        // Need to set state to Ready to get a GL context
        gtksink.set_state(gst::State::Ready).unwrap();

        let paintable = gtksink.property::<gdk::Paintable>("paintable");
        imp.picture.set_paintable(Some(&paintable));

        let sink = if paintable
            .property::<Option<gdk::GLContext>>("gl-context")
            .is_some()
        {
            gst::ElementFactory::make("glsinkbin")
                .property("sink", &gtksink)
                .build()
                .unwrap()
        } else {
            let sink = gst::Bin::default();
            let convert = gst::ElementFactory::make("videoconvert").build().unwrap();

            sink.add(&convert).unwrap();
            sink.add(&gtksink).unwrap();
            convert.link(&gtksink).unwrap();

            sink.add_pad(
                &gst::GhostPad::with_target(Some("sink"), &convert.static_pad("sink").unwrap())
                    .unwrap(),
            )
            .unwrap();

            sink.upcast()
        };

        decodebin.connect_pad_added(clone!(@weak sink => move |_, src_pad| {
            let sink_pad = sink.static_pad("sink").unwrap();
            if !sink_pad.is_linked() {
                src_pad.link(&sink_pad).unwrap();
            }
        }));

        pipeline.add_many(&[&src, &decodebin, &sink]).unwrap();

        src.link(&decodebin).unwrap();

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak pipeline => @default-return glib::Continue(false), move |_, msg| {
                use gst::MessageView;

                if let MessageView::Eos(_) = msg.view() {
                    pipeline
                        .seek_simple(gst::SeekFlags::FLUSH, gst::ClockTime::from_seconds(0))
                        .unwrap();
                }

                glib::Continue(true)
            }),
        )
        .unwrap();

        imp.pipeline.set(pipeline).unwrap();
        imp.file_src.set(src).unwrap();
    }

    fn update_content(&self, content: MessageContent, session: &Session) {
        let imp = self.imp();

        let (caption, file, aspect_ratio, minithumbnail) =
            if let MessageContent::MessageAnimation(data) = content {
                imp.indicator.set_label("GIF");
                imp.is_animation.set(true);
                (
                    data.caption,
                    data.animation.animation,
                    data.animation.width as f64 / data.animation.height as f64,
                    data.animation.minithumbnail,
                )
            } else if let MessageContent::MessageVideo(data) = content {
                self.update_remaining_time(data.video.duration as i64);
                imp.is_animation.set(false);
                (
                    data.caption,
                    data.video.video,
                    data.video.width as f64 / data.video.height as f64,
                    data.video.minithumbnail,
                )
            } else {
                unreachable!();
            };

        let caption = parse_formatted_text(caption);
        imp.message_bubble.set_label(caption);

        imp.picture.set_aspect_ratio(aspect_ratio);

        if file.local.is_downloading_completed {
            self.load_video(&file.local.path);
        } else {
            imp.picture.set_paintable(
                minithumbnail
                    .and_then(|m| {
                        gdk::Texture::from_bytes(&glib::Bytes::from_owned(glib::base64_decode(
                            &m.data,
                        )))
                        .ok()
                    })
                    .as_ref(),
            );

            let file_id = file.id;
            spawn(clone!(@weak self as obj, @weak session => async move {
                obj.download_video(file_id, &session).await;
            }));
        }
    }

    async fn download_video(&self, file_id: i32, session: &Session) {
        match session.download_file(file_id).await {
            Ok(file) => {
                self.load_video(&file.local.path);
            }
            Err(e) => {
                log::warn!("Failed to download a video: {e:?}");
            }
        }
    }

    fn load_video(&self, path: &str) {
        let imp = self.imp();
        let pipeline = imp.pipeline.get().unwrap();

        imp.file_src.get().unwrap().set_property("location", path);

        pipeline.set_state(gst::State::Playing).unwrap();

        // if !imp.is_animation.get() {
        //     media.connect_timestamp_notify(clone!(@weak self as obj => move |media| {
        //         let time = (media.duration() - media.timestamp()) / i64::pow(10, 6);
        //         obj.update_remaining_time(time);
        //     }));
        // }
    }

    fn update_remaining_time(&self, time: i64) {
        let imp = self.imp();
        let seconds = time % 60;
        let minutes = (time % (60 * 60)) / 60;
        let hours = time / (60 * 60);

        if hours > 0 {
            imp.indicator
                .set_label(&format!("{hours}:{minutes:02}:{seconds:02}"));
        } else {
            imp.indicator.set_label(&format!("{minutes}:{seconds:02}"));
        }
    }
}
