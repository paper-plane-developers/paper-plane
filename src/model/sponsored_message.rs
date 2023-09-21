use std::cell::OnceCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::SponsoredMessage)]
    pub(crate) struct SponsoredMessage {
        #[property(get, set, construct_only)]
        pub(super) message_id: OnceCell<i64>,
        #[property(get, set, construct_only)]
        pub(super) content: OnceCell<model::BoxedMessageContent>,
        #[property(get, set, construct_only)]
        pub(super) sponsor_chat: glib::WeakRef<model::Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SponsoredMessage {
        const NAME: &'static str = "SponsoredMessage";
        type Type = super::SponsoredMessage;
    }

    impl ObjectImpl for SponsoredMessage {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub(crate) struct SponsoredMessage(ObjectSubclass<imp::SponsoredMessage>);
}

impl SponsoredMessage {
    fn new(sponsored_message: &tdlib::types::SponsoredMessage, sponsor_chat: &model::Chat) -> Self {
        glib::Object::builder()
            .property("message-id", sponsored_message.message_id)
            .property(
                "content",
                model::BoxedMessageContent(sponsored_message.clone().content),
            )
            .property("sponsor-chat", sponsor_chat)
            .build()
    }

    pub(crate) async fn request(
        chat_id: i64,
        session: &model::ClientStateSession,
    ) -> Result<Option<Self>, tdlib::types::Error> {
        let tdlib::enums::SponsoredMessages::SponsoredMessages(sponsored_messages) =
            tdlib::functions::get_chat_sponsored_messages(chat_id, session.client_().id()).await?;

        // TODO: Support multiple sponsored messages
        if let Some(sponsored_message) = sponsored_messages.messages.first() {
            Ok(Some(Self::new(
                sponsored_message,
                &session.chat(sponsored_message.sponsor_chat_id),
            )))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn sponsor_chat_(&self) -> model::Chat {
        self.sponsor_chat().unwrap()
    }
}
