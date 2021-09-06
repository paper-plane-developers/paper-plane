use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdgrand::enums::Update;

use crate::session::Avatar;
use crate::Session;

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct User {
        pub id: Cell<i32>,
        pub first_name: RefCell<String>,
        pub last_name: RefCell<String>,
        pub avatar: OnceCell<Avatar>,
        pub session: OnceCell<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for User {
        const NAME: &'static str = "User";
        type Type = super::User;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for User {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int(
                        "id",
                        "Id",
                        "The id of this user",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "first-name",
                        "First Name",
                        "The first name of this user",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "last-name",
                        "Last Name",
                        "The last name of this user",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "avatar",
                        "Avatar",
                        "The avatar of this chat",
                        Avatar::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
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
                "first-name" => {
                    let first_name = value.get().unwrap();
                    self.first_name.replace(first_name);
                }
                "last-name" => {
                    let last_name = value.get().unwrap();
                    self.last_name.replace(last_name);
                }
                "avatar" => {
                    self.avatar.set(value.get().unwrap()).unwrap();
                }
                "session" => {
                    self.session.set(value.get().unwrap()).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "first-name" => self.first_name.borrow().to_value(),
                "last-name" => self.last_name.borrow().to_value(),
                "avatar" => obj.avatar().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct User(ObjectSubclass<imp::User>);
}

impl User {
    pub fn new(id: i32, session: &Session) -> Self {
        let avatar = Avatar::new(session);

        glib::Object::new(&[("id", &id), ("avatar", &avatar), ("session", session)])
            .expect("Failed to create User")
    }

    pub fn handle_update(&self, update: Update) {
        if let Update::User(data) = update {
            self.set_first_name(data.user.first_name);
            self.set_last_name(data.user.last_name);
            self.avatar()
                .update_from_user_photo(data.user.profile_photo);
        }
    }

    pub fn id(&self) -> i32 {
        self.property("id").unwrap().get().unwrap()
    }

    pub fn first_name(&self) -> String {
        self.property("first-name").unwrap().get().unwrap()
    }

    fn set_first_name(&self, first_name: String) {
        if self.first_name() != first_name {
            self.set_property("first-name", &first_name).unwrap();
        }
    }

    pub fn last_name(&self) -> String {
        self.property("last-name").unwrap().get().unwrap()
    }

    fn set_last_name(&self, last_name: String) {
        if self.last_name() != last_name {
            self.set_property("last-name", &last_name).unwrap();
        }
    }

    pub fn avatar(&self) -> &Avatar {
        let self_ = imp::User::from_instance(self);
        self_.avatar.get().unwrap()
    }

    pub fn session(&self) -> &Session {
        let self_ = imp::User::from_instance(self);
        self_.session.get().unwrap()
    }
}
