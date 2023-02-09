use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};
use tdlib::enums::MessageContent;
use tdlib::types::File;

use crate::session::content::message_row::{
    MediaPicture, MessageBase, MessageBaseImpl, MessageBubble,
};
use crate::tdlib::Message;
use crate::utils::parse_formatted_text;
use crate::Session;

use super::base::MessageBaseExt;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="MessageVideo" parent="MessageBase">
        <child>
          <object class="MessageBubble" id="message_bubble">
            <style>
              <class name="media"/>
            </style>
            <property name="prefix">
              <object class="GtkOverlay">
                <child>
                  <object class="MessageMediaPicture" id="picture"/>
                </child>
                <child type="overlay">
                  <object class="GtkLabel" id="indicator">
                    <property name="halign">start</property>
                    <property name="valign">start</property>
                    <style>
                      <class name="osd-indicator"/>
                    </style>
                  </object>
                </child>
              </object>
            </property>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    pub(crate) struct MessageVideo {
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) message: RefCell<Option<Message>>,
        pub(super) is_animation: Cell<bool>,
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
    }

    impl WidgetImpl for MessageVideo {}
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
            self.load_video_from_path(&file.local.path);
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

            self.download_video(file.id, session);
        }
    }

    fn download_video(&self, file_id: i32, session: &Session) {
        let (sender, receiver) = glib::MainContext::sync_channel::<File>(Default::default(), 5);

        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return glib::Continue(false), move |file| {
                if file.local.is_downloading_completed {
                    obj.load_video_from_path(&file.local.path);
                }

                glib::Continue(true)
            }),
        );

        session.download_file(file_id, sender);
    }

    fn load_video_from_path(&self, path: &str) {
        let imp = self.imp();

        let media = gtk::MediaFile::for_filename(path);
        media.set_muted(true);
        media.set_loop(true);
        media.play();

        if !imp.is_animation.get() {
            media.connect_timestamp_notify(clone!(@weak self as obj => move |media| {
                let time = (media.duration() - media.timestamp()) / i64::pow(10, 6);
                obj.update_remaining_time(time);
            }));
        }

        imp.picture.set_paintable(Some(&media));
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
