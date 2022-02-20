use gtk::{glib, subclass::prelude::*};
use tdgrand::enums;

use crate::session::chat::{Chat, MessageSender};

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedChatActionType")]
pub struct BoxedChatActionType(pub enums::ChatAction);

mod imp {
    use super::*;

    use gtk::glib::WeakRef;
    use gtk::prelude::{StaticType, ToValue};
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;

    #[derive(Debug, Default)]
    pub struct ChatAction {
        pub type_: OnceCell<BoxedChatActionType>,
        pub sender: OnceCell<MessageSender>,
        pub chat: WeakRef<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatAction {
        const NAME: &'static str = "ChatAction";
        type Type = super::ChatAction;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ChatAction {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::new(
                        "type",
                        "Type",
                        "The type of this chat action",
                        BoxedChatActionType::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "sender",
                        "Sender",
                        "The sender of this chat action",
                        MessageSender::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "chat",
                        "Chat",
                        "The chat relative to this chat action",
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
                "type" => self.type_.set(value.get().unwrap()).unwrap(),
                "sender" => self.sender.set(value.get().unwrap()).unwrap(),
                "chat" => self.chat.set(Some(&value.get().unwrap())),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "type" => obj.type_().to_value(),
                "sender" => obj.sender().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct ChatAction(ObjectSubclass<imp::ChatAction>);
}

impl ChatAction {
    pub fn new(type_: enums::ChatAction, sender: &enums::MessageSender, chat: &Chat) -> Self {
        glib::Object::new(&[
            ("type", &BoxedChatActionType(type_)),
            (
                "sender",
                &MessageSender::from_td_object(sender, &chat.session()),
            ),
            ("chat", chat),
        ])
        .expect("Failed to create ChatAction")
    }

    pub fn type_(&self) -> &BoxedChatActionType {
        self.imp().type_.get().unwrap()
    }

    pub fn sender(&self) -> &MessageSender {
        self.imp().sender.get().unwrap()
    }

    pub fn chat(&self) -> Chat {
        self.imp().chat.upgrade().unwrap()
    }
}
