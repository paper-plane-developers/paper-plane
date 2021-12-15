use gtk::{glib, prelude::*, subclass::prelude::*};
use tdgrand::types::SponsoredMessage as TelegramSponsoredMessage;

use crate::session::{chat::BoxedMessageContent, Chat};

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::{sync::Lazy, unsync::OnceCell};
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub struct SponsoredMessage {
        pub id: Cell<i32>,
        pub content: OnceCell<BoxedMessageContent>,
        pub sponsor_chat: WeakRef<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SponsoredMessage {
        const NAME: &'static str = "ChatSponsoredMessage";
        type Type = super::SponsoredMessage;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for SponsoredMessage {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int(
                        "id",
                        "Id",
                        "The id of this sponsored message",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "content",
                        "Content",
                        "The content of this sponsored message",
                        BoxedMessageContent::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "sponsor-chat",
                        "Sponsor Chat",
                        "The chat relative to this sponsored message",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "id" => self.id.set(value.get().unwrap()),
                "content" => self.content.set(value.get().unwrap()).unwrap(),
                "sponsor-chat" => self.sponsor_chat.set(Some(&value.get().unwrap())),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "content" => obj.content().to_value(),
                "sponsor-chat" => obj.sponsor_chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SponsoredMessage(ObjectSubclass<imp::SponsoredMessage>);
}

impl SponsoredMessage {
    pub fn new(message: TelegramSponsoredMessage, sponsor_chat: &Chat) -> Self {
        let content = BoxedMessageContent(message.content);
        glib::Object::new(&[
            ("id", &message.id),
            ("content", &content),
            ("sponsor-chat", sponsor_chat),
        ])
        .expect("Failed to create SponsoredMessage")
    }

    pub fn id(&self) -> i32 {
        let self_ = imp::SponsoredMessage::from_instance(self);
        self_.id.get()
    }

    pub fn content(&self) -> &BoxedMessageContent {
        let self_ = imp::SponsoredMessage::from_instance(self);
        self_.content.get().unwrap()
    }

    pub fn sponsor_chat(&self) -> Chat {
        let self_ = imp::SponsoredMessage::from_instance(self);
        self_.sponsor_chat.upgrade().unwrap()
    }
}
