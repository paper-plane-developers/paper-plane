use gtk::{glib, prelude::*, subclass::prelude::*};
use tdgrand::enums::{MessageContent, MessageSender as TelegramMessageSender, Update};
use tdgrand::types::Message as TelegramMessage;

use crate::session::{Chat, User};

#[derive(Clone, Debug, PartialEq, glib::GBoxed)]
#[gboxed(type_name = "BoxedMessageContent")]
pub struct BoxedMessageContent(pub MessageContent);

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "MessageSender")]
pub enum MessageSender {
    User(User),
    Chat(Chat),
}
impl MessageSender {
    pub fn as_user(&self) -> Option<&User> {
        match self {
            MessageSender::User(user) => Some(user),
            _ => None,
        }
    }
}

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct Message {
        pub id: Cell<i64>,
        pub sender: OnceCell<MessageSender>,
        pub is_outgoing: Cell<bool>,
        pub date: Cell<i32>,
        pub content: RefCell<Option<BoxedMessageContent>>,
        pub chat: OnceCell<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Message {
        const NAME: &'static str = "ChatMessage";
        type Type = super::Message;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Message {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int64(
                        "id",
                        "Id",
                        "The id of this message",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "sender",
                        "Sender",
                        "The sender of this message",
                        MessageSender::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-outgoing",
                        "Is Outgoing",
                        "Whether this message is outgoing or not",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_int(
                        "date",
                        "Date",
                        "The point in time when this message was sent",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "content",
                        "Content",
                        "The content of this message",
                        BoxedMessageContent::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpec::new_object(
                        "chat",
                        "Chat",
                        "The chat relative to this message",
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
                "sender" => {
                    let sender = value.get::<MessageSender>().unwrap();
                    self.sender.set(sender).unwrap();
                }
                "is-outgoing" => self.is_outgoing.set(value.get().unwrap()),
                "date" => self.date.set(value.get().unwrap()),
                "content" => {
                    let content = value.get().unwrap();
                    self.content.replace(Some(content));
                }
                "chat" => self.chat.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "is-outgoing" => obj.is_outgoing().to_value(),
                "date" => obj.date().to_value(),
                "content" => self.content.borrow().as_ref().unwrap().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Message(ObjectSubclass<imp::Message>);
}

impl Message {
    pub fn new(message: TelegramMessage, chat: &Chat) -> Self {
        let content = BoxedMessageContent(message.content);
        let sender = match message.sender {
            TelegramMessageSender::User(data) => {
                let user = chat.session().user_list().get_or_create_user(data.user_id);
                MessageSender::User(user)
            }
            TelegramMessageSender::Chat(data) => {
                let chat = chat.session().chat_list().get_chat(data.chat_id).unwrap();
                MessageSender::Chat(chat)
            }
        };

        glib::Object::new(&[
            ("id", &message.id),
            ("sender", &sender),
            ("is-outgoing", &message.is_outgoing),
            ("date", &message.date),
            ("content", &content),
            ("chat", chat),
        ])
        .expect("Failed to create Message")
    }

    pub fn handle_update(&self, update: Update) {
        if let Update::MessageContent(data) = update {
            let new_content = BoxedMessageContent(data.new_content);
            self.set_content(new_content);
        }
    }

    pub fn id(&self) -> i64 {
        let self_ = imp::Message::from_instance(self);
        self_.id.get()
    }

    pub fn sender(&self) -> &MessageSender {
        let self_ = imp::Message::from_instance(self);
        self_.sender.get().unwrap()
    }

    pub fn is_outgoing(&self) -> bool {
        let self_ = imp::Message::from_instance(self);
        self_.is_outgoing.get()
    }

    pub fn date(&self) -> i32 {
        let self_ = imp::Message::from_instance(self);
        self_.date.get()
    }

    pub fn content(&self) -> BoxedMessageContent {
        self.property("content").unwrap().get().unwrap()
    }

    fn set_content(&self, content: BoxedMessageContent) {
        if self.content() != content {
            self.set_property("content", &content).unwrap();
        }
    }

    pub fn chat(&self) -> &Chat {
        let self_ = imp::Message::from_instance(self);
        self_.chat.get().unwrap()
    }

    pub fn sender_name_expression(&self) -> gtk::Expression {
        match self.sender() {
            MessageSender::User(user) => {
                let user_expression = gtk::ConstantExpression::new(user);
                User::full_name_expression(&user_expression)
            }
            MessageSender::Chat(chat) => chat.title_expression(),
        }
    }
}
