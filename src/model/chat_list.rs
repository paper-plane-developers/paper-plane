use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::BTreeMap;

use glib::clone;
use glib::Properties;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::ChatList)]
    pub(crate) struct ChatList {
        // order -> item
        pub(super) list: RefCell<BTreeMap<i64, model::ChatListItem>>,
        #[property(get, set, construct_only)]
        pub(super) session: glib::WeakRef<model::ClientStateSession>,
        #[property(get, set)]
        pub(super) unread_count: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatList {
        const NAME: &'static str = "ChatList";
        type Type = super::ChatList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ChatList {
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

    impl ListModelImpl for ChatList {
        fn item_type(&self) -> glib::Type {
            model::ChatListItem::static_type()
        }

        fn n_items(&self) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .iter()
                .nth(position as usize)
                .map(|(_, c)| c.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatList(ObjectSubclass<imp::ChatList>)
        @implements gio::ListModel;
}

impl From<&model::ClientStateSession> for ChatList {
    fn from(session: &model::ClientStateSession) -> Self {
        glib::Object::builder().property("session", session).build()
    }
}

impl ChatList {
    pub(crate) fn session_(&self) -> model::ClientStateSession {
        self.session().unwrap()
    }

    pub(crate) fn fetch(&self) {
        utils::spawn(clone!(@weak self as obj => async move {
            let result = tdlib::functions::load_chats(None, 20, obj.session_().client_().id())
                .await;

            if let Err(err) = result {
                // Error 404 means that all chats have been loaded
                if err.code != 404 {
                    log::error!("Received an error for LoadChats: {}", err.code);
                }
            } else {
                obj.fetch();
            }
        }));
    }

    pub(crate) fn find_chat_item(&self, chat_id: i64) -> Option<model::ChatListItem> {
        self.imp()
            .list
            .borrow()
            .iter()
            .find(|(_, item)| item.chat().unwrap().id() == chat_id)
            .map(|(_, item)| item)
            .cloned()
    }

    pub(crate) fn update_chat_position(
        &self,
        chat: &model::Chat,
        position: &tdlib::types::ChatPosition,
    ) {
        let imp = self.imp();
        let mut list = imp.list.borrow_mut();

        match find_chat_item_position(&list, chat.id()) {
            Some((old_pos, old_order)) => {
                let item = list.remove(&old_order).unwrap();

                drop(list);
                self.items_changed(old_pos as u32, 1, 0);

                if position.order != 0 {
                    item.update(position);

                    self.insert_item(imp.list.borrow_mut(), item, position.order);
                }
            }
            None => self.insert_item(
                list,
                model::ChatListItem::new(chat, position),
                position.order,
            ),
        }
    }

    fn insert_item(
        &self,
        mut list: RefMut<BTreeMap<i64, model::ChatListItem>>,
        item: model::ChatListItem,
        order: i64,
    ) {
        let client_id = item.chat().unwrap().id();

        // Invert the sign to have a descending order
        list.insert(-order, item);

        let position = find_chat_item_position(&list, client_id).unwrap().0;

        drop(list);
        self.items_changed(position as u32, 0, 1);
    }
}

fn find_chat_item_position(
    list: &RefMut<BTreeMap<i64, model::ChatListItem>>,
    chat_id: i64,
) -> Option<(usize, i64)> {
    list.iter()
        .enumerate()
        .find(|(_, (_, item))| item.chat().unwrap().id() == chat_id)
        .map(|(pos, (order, _))| (pos, *order))
}
