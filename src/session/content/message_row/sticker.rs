use adw::prelude::*;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use tdlib::enums::MessageContent;

use crate::session::content::message_row::{
    MessageBase, MessageBaseImpl, MessageIndicators, MessageReply, StickerPicture,
};
use crate::tdlib::Message;
use crate::utils::{decode_image_from_path, spawn};
use crate::Session;

use super::base::MessageBaseExt;

const MAX_REPLY_CHAR_WIDTH: i32 = 18;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-sticker.ui")]
    pub(crate) struct MessageSticker {
        pub(super) message: RefCell<Option<Message>>,
        #[template_child]
        pub(super) picture: TemplateChild<StickerPicture>,
        #[template_child]
        pub(super) sticker_overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub(super) indicators: TemplateChild<MessageIndicators>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageSticker {
        const NAME: &'static str = "MessageSticker";
        type Type = super::MessageSticker;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("messagesticker");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageSticker {
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

    impl WidgetImpl for MessageSticker {}
    impl MessageBaseImpl for MessageSticker {}
}

glib::wrapper! {
    pub(crate) struct MessageSticker(ObjectSubclass<imp::MessageSticker>)
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessageSticker {
    type Message = Message;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        imp.message.replace(Some(message));

        let message_ref = imp.message.borrow();
        let message = message_ref.as_ref().unwrap();

        imp.indicators.set_message(message.clone().upcast());

        if message.reply_to_message_id() != 0 {
            let reply = MessageReply::new(
                message.chat(),
                message.reply_to_message_id(),
                message.is_outgoing(),
            );
            reply.set_valign(gtk::Align::Start);
            reply.set_max_char_width(MAX_REPLY_CHAR_WIDTH);

            // FIXME: Do not show message reply when message is being deleted
            // Sticker and the reply should be at the opposite sides of the box
            if message.is_outgoing() {
                reply.insert_before(self, Some(&imp.sticker_overlay.get()));
            } else {
                reply.insert_after(self, Some(&imp.sticker_overlay.get()));
            }
        }
        imp.picture.set_texture(None);

        if let MessageContent::MessageSticker(data) = message.content().0 {
            self.imp()
                .picture
                .set_aspect_ratio(data.sticker.width as f64 / data.sticker.height as f64);

            if data.sticker.sticker.local.is_downloading_completed {
                self.load_sticker(data.sticker.sticker.local.path);
            } else {
                let file_id = data.sticker.sticker.id;
                let session = message.chat().session();

                spawn(clone!(@weak self as obj, @weak session => async move {
                    obj.download_sticker(file_id, &session).await;
                }));
            }
        }

        self.notify("message");
    }
}

impl MessageSticker {
    async fn download_sticker(&self, file_id: i32, session: &Session) {
        match session.download_file(file_id).await {
            Ok(file) => {
                self.load_sticker(file.local.path);
            }
            Err(e) => {
                log::warn!("Failed to download a sticker: {e:?}");
            }
        }
    }

    fn load_sticker(&self, path: String) {
        let message_id = self.message().id();

        spawn(clone!(@weak self as obj => async move {
            let result = gio::spawn_blocking(move || decode_image_from_path(&path))
                .await
                .unwrap();

            // Check if the current message id is the same as the one at
            // the time of the request. It may be changed because of the
            // ListView recycling while decoding the image.
            if obj.message().id() != message_id {
                return;
            }

            match result {
                Ok(texture) => {
                    obj.imp().picture.set_texture(Some(texture.upcast()));
                }
                Err(e) => {
                    log::warn!("Error decoding a sticker: {e:?}");
                }
            }
        }));
    }
}
