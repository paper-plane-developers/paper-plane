use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tdgrand::enums::Update;

use crate::session::User;

mod imp {
    use super::*;
    use indexmap::IndexMap;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct UserList {
        pub list: RefCell<IndexMap<i32, User>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserList {
        const NAME: &'static str = "UserList";
        type Type = super::UserList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for UserList {}

    impl ListModelImpl for UserList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            User::static_type()
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
    pub struct UserList(ObjectSubclass<imp::UserList>)
        @implements gio::ListModel;
}

impl Default for UserList {
    fn default() -> Self {
        Self::new()
    }
}

impl UserList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create UserList")
    }

    pub fn insert_user(&self, user: User) {
        {
            let priv_ = imp::UserList::from_instance(self);
            let mut list = priv_.list.borrow_mut();
            list.insert(user.id(), user);
        }

        self.item_added();
    }

    pub fn get_or_create_user(&self, user_id: i32) -> User {
        let priv_ = imp::UserList::from_instance(self);
        if let Some(index) = priv_.list.borrow().get_index_of(&user_id) {
            if let Some(item) = self.item(index as u32) {
                return item.downcast().unwrap();
            }
        }

        let user = User::new(user_id);
        self.insert_user(user);
        self.get_or_create_user(user_id)
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::User(ref update_) => {
                let user = self.get_or_create_user(update_.user.id);
                user.handle_update(update);
            }
            _ => {}
        }
    }

    fn item_added(&self) {
        let priv_ = imp::UserList::from_instance(self);
        let list = priv_.list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1);
    }
}
