use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::Entry;
use tdgrand::enums::Update;

use crate::session::User;
use crate::Session;

mod imp {
    use super::*;
    use indexmap::IndexMap;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct UserList {
        pub list: RefCell<IndexMap<i64, User>>,
        pub session: OnceCell<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserList {
        const NAME: &'static str = "UserList";
        type Type = super::UserList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for UserList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
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
                "session" => self.session.set(value.get().unwrap()).unwrap(),

                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }
    }

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
                .get_index(position as usize)
                .map(|(_, u)| u.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct UserList(ObjectSubclass<imp::UserList>)
        @implements gio::ListModel;
}

impl UserList {
    pub fn new(session: &Session) -> Self {
        glib::Object::new(&[("session", session)]).expect("Failed to create UserList")
    }

    pub fn insert_user(&self, user: User) {
        {
            let self_ = imp::UserList::from_instance(self);
            let mut list = self_.list.borrow_mut();
            list.insert(user.id(), user);
        }

        self.item_added();
    }

    /// Return the `User` of the specified `id`. Panics if the user is not present.
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an `id` returned by TDLib, it should be expected that the
    /// relative `User` exists in the list.
    pub fn get(&self, id: i64) -> User {
        let self_ = imp::UserList::from_instance(self);
        self_
            .list
            .borrow()
            .get(&id)
            .expect("Failed to get expected User")
            .to_owned()
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::User(data) => {
                let self_ = imp::UserList::from_instance(self);
                let mut list = self_.list.borrow_mut();

                match list.entry(data.user.id) {
                    Entry::Occupied(entry) => entry.get().handle_update(Update::User(data)),
                    Entry::Vacant(entry) => {
                        let user = User::from_td_object(data.user, self.session());
                        entry.insert(user);

                        drop(list);
                        self.item_added();
                    }
                }
            }
            Update::UserStatus(ref data) => self.get(data.user_id).handle_update(update),
            _ => {}
        }
    }

    fn item_added(&self) {
        let self_ = imp::UserList::from_instance(self);
        let list = self_.list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1);
    }

    pub fn session(&self) -> &Session {
        let self_ = imp::UserList::from_instance(self);
        self_.session.get().unwrap()
    }
}
