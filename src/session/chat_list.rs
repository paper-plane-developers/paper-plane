use crate::session::Chat;
use crate::utils::do_async;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tdgrand::{enums, functions};

mod imp {
    use super::*;
    use indexmap::IndexMap;
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

    impl ObjectImpl for ChatList {}

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

    pub fn load(&self, client_id: i32) {
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::get_chats(client_id, enums::ChatList::Main, i64::MAX, 0, i32::MAX).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Ok(enums::Chats::Chats(chats)) = result {
                    for chat_id in chats.chat_ids {
                        obj.insert(chat_id, client_id);
                    }
                }
            }),
        );
    }

    fn item_added(&self) {
        let priv_ = imp::ChatList::from_instance(self);
        let list = priv_.list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1 as u32);
    }

    fn insert(&self, chat_id: i64, client_id: i32) {
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::get_chat(client_id, chat_id).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Ok(enums::Chat::Chat(chat)) = result {
                    {
                        let priv_ = imp::ChatList::from_instance(&obj);
                        let mut list = priv_.list.borrow_mut();
                        list.insert(chat.id, Chat::new(chat));
                    }

                    obj.item_added();
                }
            }),
        );
    }
}
