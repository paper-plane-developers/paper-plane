use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::types::Error as TdError;
use tdlib::{enums, functions};

use crate::tdlib::{BoxedMessageContent, Chat};
use crate::Session;

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub(crate) struct SponsoredMessage {
        pub(super) message_id: Cell<i64>,
        pub(super) content: OnceCell<BoxedMessageContent>,
        pub(super) sponsor_chat: WeakRef<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SponsoredMessage {
        const NAME: &'static str = "SponsoredMessage";
        type Type = super::SponsoredMessage;
    }

    impl ObjectImpl for SponsoredMessage {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::new(
                        "message-id",
                        "Message Id",
                        "The id of this message",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        "content",
                        "Content",
                        "The content of this message",
                        BoxedMessageContent::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        "sponsor-chat",
                        "Sponsor Chat",
                        "The chat relative to this sponsored message",
                        Chat::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "message-id" => obj.message_id().to_value(),
                "content" => obj.content().to_value(),
                "sponsor-chat" => obj.sponsor_chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct SponsoredMessage(ObjectSubclass<imp::SponsoredMessage>);
}

impl SponsoredMessage {
    pub(crate) async fn request(chat_id: i64, session: &Session) -> Result<Self, TdError> {
        let enums::SponsoredMessage::SponsoredMessage(td_sponsored_message) =
            functions::get_chat_sponsored_message(chat_id, session.client_id()).await?;

        let sponsored_message: SponsoredMessage = glib::Object::builder().build();
        let imp = sponsored_message.imp();

        let content = BoxedMessageContent(td_sponsored_message.content);
        let sponsor_chat = session
            .chat_list()
            .get(td_sponsored_message.sponsor_chat_id);

        imp.message_id.set(td_sponsored_message.message_id);
        imp.content.set(content).unwrap();
        imp.sponsor_chat.set(Some(&sponsor_chat));

        Ok(sponsored_message)
    }

    pub(crate) fn message_id(&self) -> i64 {
        self.imp().message_id.get()
    }

    pub(crate) fn content(&self) -> &BoxedMessageContent {
        self.imp().content.get().unwrap()
    }

    pub(crate) fn sponsor_chat(&self) -> Chat {
        self.imp().sponsor_chat.upgrade().unwrap()
    }
}
