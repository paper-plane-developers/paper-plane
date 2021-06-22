use crate::session::{chat::stringify_message, Chat};
use crate::RUNTIME;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tdgrand::{
    enums,
    functions,
    types::Chat as TelegramChat,
};

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use indexmap::IndexMap;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct ChatList {
        pub list: RefCell<IndexMap<i64, Chat>>,
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
                vec![Signal::builder(
                    "positions-changed",
                    &[],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
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
                .values()
                .nth(position as usize)
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct ChatList(ObjectSubclass<imp::ChatList>)
        @implements gio::ListModel;
}

impl Default for ChatList {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChatList")
    }

    pub fn fetch(&self, client_id: i32) {
        RUNTIME.spawn(async move {
            functions::get_chats(client_id, enums::ChatList::Main, i64::MAX, 0, i32::MAX).await.unwrap();
        });
    }

    pub fn handle_update(&self, update: enums::Update) {
        let priv_ = imp::ChatList::from_instance(self);

        match update {
            enums::Update::NewChat(update) => {
                self.insert_chat(update.chat);
            },
            enums::Update::ChatTitle(update) => {
                if let Some(chat) = priv_.list.borrow().get(&update.chat_id) {
                    chat.set_title(update.title);
                }
            },
            enums::Update::ChatLastMessage(update) => {
                if let Some(chat) = priv_.list.borrow().get(&update.chat_id) {
                    let message = stringify_message(update.last_message);
                    chat.set_last_message(message);

                    for position in update.positions {
                        if let enums::ChatList::Main = position.list {
                            self.update_chat_order(chat, position.order);
                            break;
                        }
                    }
                }
            },
            enums::Update::ChatPosition(update) => {
                if let enums::ChatList::Main = update.position.list {
                    let list = priv_.list.borrow();
                    if let Some(chat) = list.get(&update.chat_id) {
                        self.update_chat_order(chat, update.position.order);
                    }
                }
            },
            enums::Update::ChatReadInbox(update) => {
                if let Some(chat) = priv_.list.borrow().get(&update.chat_id) {
                    chat.set_unread_count(update.unread_count);
                }
            },
            _ => (),
        }
    }

    fn insert_chat(&self, chat: TelegramChat) {
        {
            let priv_ = imp::ChatList::from_instance(self);
            let mut list = priv_.list.borrow_mut();
            list.insert(chat.id, Chat::new(chat));
        }

        self.item_added();
    }

    fn item_added(&self) {
        let priv_ = imp::ChatList::from_instance(self);
        let list = priv_.list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1 as u32);
    }

    fn update_chat_order(&self, chat: &Chat, order: i64) {
        if chat.order() != order {
            chat.set_order(order);
            self.emit_by_name("positions-changed", &[]).unwrap();
        }
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
