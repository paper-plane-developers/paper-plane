use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::Entry;
use tdlib::enums::Update;

use crate::tdlib::User;
use crate::Session;

mod imp {
    use super::*;
    use glib::WeakRef;
    use indexmap::IndexMap;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub(crate) struct UserList {
        pub(super) list: RefCell<IndexMap<i64, User>>,
        pub(super) session: WeakRef<Session>,
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
                    glib::ParamFlags::READABLE,
                )]
            });

            PROPERTIES.as_ref()
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
    pub(crate) struct UserList(ObjectSubclass<imp::UserList>)
        @implements gio::ListModel;
}

impl UserList {
    pub(crate) fn new(session: &Session) -> Self {
        let user_list: UserList = glib::Object::new(&[]).expect("Failed to create UserList");
        user_list.imp().session.set(Some(session));
        user_list
    }

    /// Return the `User` of the specified `id`. Panics if the user is not present.
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an `id` returned by TDLib, it should be expected that the
    /// relative `User` exists in the list.
    pub(crate) fn get(&self, id: i64) -> User {
        self.try_get(id).expect("Failed to get expected User")
    }

    pub(crate) fn try_get(&self, id: i64) -> Option<User> {
        self.imp().list.borrow().get(&id).cloned()
    }

    pub(crate) fn handle_update(&self, update: Update) {
        match update {
            Update::User(data) => {
                let mut list = self.imp().list.borrow_mut();

                match list.entry(data.user.id) {
                    Entry::Occupied(entry) => entry.get().handle_update(Update::User(data)),
                    Entry::Vacant(entry) => {
                        let user = User::from_td_object(data.user, &self.session());
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
        let list = self.imp().list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1);
    }

    pub(crate) fn session(&self) -> Session {
        self.imp().session.upgrade().unwrap()
    }
}
