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
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="ContentMessageVideo" parent="ContentMessageBase">
        <child>
          <object class="MessageBubble" id="message_bubble">
            <style>
              <class name="media"/>
            </style>
            <property name="prefix">
              <object class="GtkOverlay">
                <child>
                  <object class="ContentMediaPicture" id="picture"/>
                </child>
                <child type="overlay">
                  <object class="GtkLabel">
                    <property name="label">GIF</property>
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
        #[template_child]
        pub(super) message_bubble: TemplateChild<MessageBubble>,
        #[template_child]
        pub(super) picture: TemplateChild<MediaPicture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageVideo {
        const NAME: &'static str = "ContentMessageVideo";
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
                vec![glib::ParamSpecObject::new(
                    "message",
                    "Message",
                    "The message represented by this row",
                    Message::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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

        if let MessageContent::MessageAnimation(data) = content {
            let caption = parse_formatted_text(data.caption);
            imp.message_bubble.set_label(caption);

            imp.picture
                .set_aspect_ratio(data.animation.width as f64 / data.animation.height as f64);

            if data.animation.animation.local.is_downloading_completed {
                self.load_video_from_path(&data.animation.animation.local.path);
            } else {
                imp.picture.set_paintable(
                    data.animation
                        .minithumbnail
                        .and_then(|m| {
                            gdk::Texture::from_bytes(&glib::Bytes::from_owned(glib::base64_decode(
                                &m.data,
                            )))
                            .ok()
                        })
                        .as_ref(),
                );

                self.download_video(data.animation.animation.id, session);
            }
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
        let media = gtk::MediaFile::for_filename(path);
        media.set_loop(true);
        media.play();

        self.imp().picture.set_paintable(Some(&media));
    }
}
