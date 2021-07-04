use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::types::Message as TelegramMessage;

use crate::session::chat::{MessageContent, MessageSender};

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub struct Message {
        pub id: Cell<i64>,
        pub sender: OnceCell<MessageSender>,
        pub outgoing: Cell<bool>,
        pub date: Cell<i32>,
        pub content: OnceCell<MessageContent>,
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boolean(
                        "outgoing",
                        "Outgoing",
                        "Wheter this message is outgoing or not",
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
                        MessageContent::static_type(),
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
                "id" => {
                    let id = value.get().unwrap();
                    self.id.set(id);
                }
                "sender" => {
                    let sender = value.get().unwrap();
                    self.sender.set(sender).unwrap();
                }
                "outgoing" => {
                    let outgoing = value.get().unwrap();
                    self.outgoing.set(outgoing);
                }
                "date" => {
                    let date = value.get().unwrap();
                    self.date.set(date);
                }
                "content" => {
                    let content = value.get().unwrap();
                    self.content.set(content).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "sender" => self.sender.get().unwrap().to_value(),
                "outgoing" => self.outgoing.get().to_value(),
                "date" => self.date.get().to_value(),
                "content" => self.content.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Message(ObjectSubclass<imp::Message>);
}

impl Message {
    pub fn new(message: TelegramMessage, sender: MessageSender) -> Self {
        let content = MessageContent::new(message.content);
        glib::Object::new(&[
            ("id", &message.id),
            ("sender", &sender),
            ("outgoing", &message.is_outgoing),
            ("date", &message.date),
            ("content", &content),
        ])
        .expect("Failed to create Message")
    }

    pub fn id(&self) -> i64 {
        self.property("id").unwrap().get().unwrap()
    }

    pub fn sender(&self) -> MessageSender {
        self.property("sender").unwrap().get().unwrap()
    }

    pub fn outgoing(&self) -> bool {
        self.property("outgoing").unwrap().get().unwrap()
    }

    pub fn date(&self) -> i32 {
        self.property("date").unwrap().get().unwrap()
    }

    pub fn content(&self) -> MessageContent {
        self.property("content").unwrap().get().unwrap()
    }
}
