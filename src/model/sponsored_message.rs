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
        pub(super) sponsor_label: OnceCell<String>,
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
    fn new(
        sponsored_message: tdlib::types::SponsoredMessage,
        session: &model::ClientStateSession,
    ) -> Self {
        use tdlib::enums::MessageSponsorType::*;

        glib::Object::builder()
            .property("message-id", sponsored_message.message_id)
            .property(
                "content",
                model::BoxedMessageContent(sponsored_message.clone().content),
            )
            .property(
                "sponsor-label",
                match &sponsored_message.sponsor.r#type {
                    Bot(sponsor) => session.user(sponsor.bot_user_id).first_name(),
                    PublicChannel(sponsor) => session.chat(sponsor.chat_id).title(),
                    PrivateChannel(sponsor) => sponsor.title.clone(),
                    Website(sponsor) => sponsor.name.clone(),
                },
            )
            .build()
    }

    pub(crate) async fn request(
        chat_id: i64,
        session: &model::ClientStateSession,
    ) -> Result<Option<Self>, tdlib::types::Error> {
        tdlib::functions::get_chat_sponsored_messages(chat_id, session.client_().id())
            .await
            .map(
                |tdlib::enums::SponsoredMessages::SponsoredMessages(mut sponsored_messages)| {
                    sponsored_messages
                        .messages
                        .pop()
                        .map(|sponsored_message| Self::new(sponsored_message, session))
                },
            )
    }
}
