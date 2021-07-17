use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tdgrand::enums::{self, Update};
use tdgrand::functions;
use tdgrand::types::Message as TelegramMessage;

use crate::session::chat::Message;
use crate::session::Chat;
use crate::utils::do_async;

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};

    #[derive(Debug, Default)]
    pub struct History {
        pub list: RefCell<VecDeque<Message>>,
        pub message_map: RefCell<HashMap<i64, Message>>,
        pub chat: OnceCell<Chat>,
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
                vec![glib::ParamSpec::new_object(
                    "chat",
                    "Chat",
                    "The chat relative to this history",
                    Chat::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
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
                "chat" => {
                    let chat = value.get().unwrap();
                    self.chat.set(chat).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => self.chat.get().to_value(),
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
                .get(position as usize)
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct History(ObjectSubclass<imp::History>)
        @implements gio::ListModel;
}

impl History {
    pub fn new(chat: &Chat) -> Self {
        glib::Object::new(&[("chat", chat)]).expect("Failed to create History")
    }

    pub fn fetch(&self) {
        let self_ = imp::History::from_instance(self);
        let limit = 20;

        // TODO: remove this when proper automatic fetch is implemented
        if self_.list.borrow().len() >= limit {
            return;
        }

        let chat = self.chat();
        let client_id = chat.session().client_id();
        let chat_id = chat.id();
        let oldest_message_id = self_
            .list
            .borrow()
            .front()
            .map(|message| message.id())
            .unwrap_or_default();

        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::GetChatHistory::new()
                    .chat_id(chat_id)
                    .from_message_id(oldest_message_id)
                    .limit(limit as i32)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Ok(enums::Messages::Messages(result)) = result {
                    if let Some(messages) = result.messages {
                        obj.prepend(messages);
                    }
                }
            }),
        );
    }

    pub fn message_by_id(&self, id: i64) -> Option<Message> {
        let self_ = imp::History::from_instance(self);
        self_.message_map.borrow().get(&id).cloned()
    }

    pub fn handle_update(&self, update: Update) {
        let self_ = imp::History::from_instance(self);

        match update {
            Update::NewMessage(update) => {
                if !self_.message_map.borrow().contains_key(&update.message.id) {
                    self.append(update.message);
                }
            }
            Update::MessageContent(ref update_) => {
                if let Some(message) = self_.message_map.borrow().get(&update_.message_id) {
                    message.handle_update(update);
                }
            }
            _ => {}
        }
    }

    pub fn append(&self, message: TelegramMessage) {
        let self_ = imp::History::from_instance(self);
        let message = Message::new(message, &self.chat());

        self_
            .message_map
            .borrow_mut()
            .insert(message.id(), message.clone());

        self_.list.borrow_mut().push_back(message);

        let index = self_.list.borrow().len() - 1;
        self.items_changed(index as u32, 0, 1);
    }

    fn prepend(&self, messages: Vec<TelegramMessage>) {
        let self_ = imp::History::from_instance(self);
        let chat = self.chat();
        let added = messages.len();

        self_.list.borrow_mut().reserve(added);

        for message in messages {
            let message = Message::new(message, &chat);

            self_
                .message_map
                .borrow_mut()
                .insert(message.id(), message.clone());

            self_.list.borrow_mut().push_front(message);
        }

        self.items_changed(0, 0, added as u32);
    }

    pub fn chat(&self) -> Chat {
        self.property("chat").unwrap().get().unwrap()
    }
}
