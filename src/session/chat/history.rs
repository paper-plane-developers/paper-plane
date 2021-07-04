use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tdgrand::enums::MessageSender as TelegramMessageSender;
use tdgrand::enums::{self, Update};
use tdgrand::functions;
use tdgrand::types::Message as TelegramMessage;

use crate::Session;
use crate::session::chat::{Message, MessageSender};
use crate::utils::do_async;

mod imp {
    use super::*;
    use indexmap::IndexMap;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct History {
        pub list: RefCell<IndexMap<i64, Message>>,
        pub chat_id: Cell<i64>,
        pub oldest_message_id: Cell<i64>,
        pub session: RefCell<Option<Session>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for History {
        const NAME: &'static str = "ChatHistory";
        type Type = super::History;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for History {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int64(
                        "chat-id",
                        "Chat Id",
                        "The chat id relative to this chat history",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_int64(
                        "oldest-message-id",
                        "Oldest Message Id",
                        "The oldest message id of this list",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "chat-id" => {
                    let chat_id = value.get().unwrap();
                    self.chat_id.set(chat_id);
                }
                "oldest-message-id" => {
                    let oldest_message_id = value.get().unwrap();
                    obj.set_oldest_message_id(oldest_message_id);
                }
                "session" => {
                    let session = value.get().unwrap();
                    self.session.replace(session);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat-id" => obj.chat_id().to_value(),
                "oldest-message-id" => obj.oldest_message_id().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for History {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Message::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .values()
                .nth(position as usize)
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct History(ObjectSubclass<imp::History>)
        @implements gio::ListModel;
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create History")
    }

    pub fn fetch(&self) {
        let priv_ = imp::History::from_instance(self);
        let limit = 20;
        // TODO: remove this when proper automatic fetch is implemented
        if priv_.list.borrow().len() >= limit {
            return;
        }

        let client_id = self.session().client_id();
        let chat_id = self.chat_id();
        let oldest_message_id = self.oldest_message_id();

        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::GetChatHistory::new()
                    .chat_id(chat_id)
                    .from_message_id(oldest_message_id)
                    .limit(limit as i32)
                    .send(client_id).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Ok(enums::Messages::Messages(result)) = result {
                    if let Some(messages) = result.messages {
                        if let Some(oldest_message) = messages.last() {
                            obj.set_oldest_message_id(oldest_message.id);
                        }

                        let fetch_again = messages.len() < limit;
                        for message in messages {
                            obj.insert_message(message);
                        }

                        // TODO: remove this when proper automatic fetch is implemented
                        if fetch_again {
                            obj.fetch();
                        }
                    }
                }
            }),
        );
    }

    pub fn handle_update(&self, update: Update) {
        let priv_ = imp::History::from_instance(self);

        match update {
            Update::NewMessage(update) => {
                // Set this as the oldest message if we had no oldest message id
                if self.oldest_message_id() == 0 {
                    self.set_oldest_message_id(update.message.id);
                }

                self.insert_message(update.message);
            }
            Update::MessageContent(ref update_) => {
                if let Some(message) = priv_.list.borrow().get(&update_.message_id) {
                    message.handle_update(update);
                }
            }
            _ => {}
        }
    }

    fn insert_message(&self, message: TelegramMessage) {
        {
            let sender = match message.sender {
                TelegramMessageSender::User(ref sender) => {
                    let user = self
                        .session()
                        .user_list()
                        .get_or_create_user(sender.user_id);
                    MessageSender::User(user)
                },
                TelegramMessageSender::Chat(ref sender) => {
                    let chat = self
                        .session()
                        .chat_list()
                        .get_chat(sender.chat_id)
                        .unwrap();
                    MessageSender::Chat(chat)
                },
            };

            let priv_ = imp::History::from_instance(self);
            let mut list = priv_.list.borrow_mut();
            list.insert(message.id, Message::new(message, sender));
        }

        self.item_added();
    }

    fn item_added(&self) {
        let priv_ = imp::History::from_instance(self);
        let list = priv_.list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1);
    }

    pub fn chat_id(&self) -> i64 {
        let priv_ = imp::History::from_instance(self);
        priv_.chat_id.get()
    }

    pub fn oldest_message_id(&self) -> i64 {
        let priv_ = imp::History::from_instance(self);
        priv_.oldest_message_id.get()
    }

    fn set_oldest_message_id(&self, oldest_message_id: i64) {
        let priv_ = imp::History::from_instance(self);
        priv_.oldest_message_id.replace(oldest_message_id);
        self.notify("oldest-message-id");
    }

    pub fn session(&self) -> Session {
        let priv_ = imp::History::from_instance(self);
        priv_.session.borrow().as_ref().unwrap().to_owned()
    }
}
