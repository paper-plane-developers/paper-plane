use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tdgrand::types::Chat as TelegramChat;
use tdgrand::{enums::Update, functions};

use crate::session::Chat;
use crate::utils::do_async;
use crate::Session;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use indexmap::IndexMap;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct ChatList {
        pub list: RefCell<IndexMap<i64, Chat>>,
        pub session: OnceCell<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatList {
        const NAME: &'static str = "ChatList";
        type Type = super::ChatList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ChatList {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("positions-changed", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "session",
                    "Session",
                    "The session",
                    Session::static_type(),
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
                "session" => {
                    let session = value.get().unwrap();
                    self.session.set(session).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "session" => self.session.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for ChatList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Chat::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|(_, c)| c.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct ChatList(ObjectSubclass<imp::ChatList>)
        @implements gio::ListModel;
}

impl ChatList {
    pub fn new(session: &Session) -> Self {
        glib::Object::new(&[("session", session)]).expect("Failed to create ChatList")
    }

    pub fn fetch(&self, client_id: i32) {
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            functions::LoadChats::new().limit(20).send(client_id),
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    // Error 404 means that all chats have been loaded
                    if err.code != 404 {
                        log::error!("Received an error for LoadChats: {}", err.code);
                    }
                } else {
                    obj.fetch(client_id);
                }
            }),
        );
    }

    pub fn handle_update(&self, update: Update) {
        let self_ = imp::ChatList::from_instance(self);

        match update {
            Update::NewMessage(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.message.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::MessageSendSucceeded(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.message.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::MessageContent(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::NewChat(update) => {
                self.insert_chat(update.chat);
            }
            Update::ChatTitle(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::ChatPhoto(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::ChatLastMessage(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::ChatNotificationSettings(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::ChatPosition(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::ChatUnreadMentionCount(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::MessageMentionRead(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::ChatReadInbox(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::ChatDraftMessage(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            Update::DeleteMessages(ref update_) => {
                if let Some(chat) = self_.list.borrow().get(&update_.chat_id) {
                    chat.handle_update(update);
                }
            }
            _ => {}
        }
    }

    pub fn get_chat(&self, chat_id: i64) -> Option<Chat> {
        let self_ = imp::ChatList::from_instance(self);
        self_.list.borrow().get(&chat_id).cloned()
    }

    fn insert_chat(&self, chat: TelegramChat) {
        {
            let self_ = imp::ChatList::from_instance(self);
            let mut list = self_.list.borrow_mut();
            let chat_id = chat.id;
            let chat = Chat::new(chat, self.session());

            chat.connect_order_notify(clone!(@weak self as obj => move |_, _| {
                obj.emit_by_name("positions-changed", &[]).unwrap();
            }));

            list.insert(chat_id, chat);
        }

        self.item_added();
    }

    fn item_added(&self) {
        let self_ = imp::ChatList::from_instance(self);
        let list = self_.list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1);
    }

    pub fn session(&self) -> Session {
        self.property("session").unwrap().get().unwrap()
    }

    pub fn connect_positions_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("positions-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);

            None
        })
        .unwrap()
    }
}
