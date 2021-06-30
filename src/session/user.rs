use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::enums::Update;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct User {
        pub id: Cell<i32>,
        pub first_name: RefCell<String>,
        pub last_name: RefCell<String>,
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_string(
                        "last-name",
                        "Last Name",
                        "The last name of this user",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "id" => {
                    let id = value.get().unwrap();
                    self.id.set(id);
                }
                "first-name" => {
                    let first_name = value.get().unwrap();
                    obj.set_first_name(first_name);
                }
                "last-name" => {
                    let last_name = value.get().unwrap();
                    obj.set_last_name(last_name);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "first-name" => obj.first_name().to_value(),
                "last-name" => obj.last_name().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct User(ObjectSubclass<imp::User>);
}

impl User {
    pub fn new(id: i32) -> Self {
        glib::Object::new(&[("id", &id)])
            .expect("Failed to create User")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::User(update) => {
                self.set_first_name(update.user.first_name);
                self.set_last_name(update.user.last_name);
            },
            _ => (),
        }
    }

    pub fn id(&self) -> i32 {
        let priv_ = imp::User::from_instance(self);
        priv_.id.get()
    }

    pub fn first_name(&self) -> String {
        let priv_ = imp::User::from_instance(self);
        priv_.first_name.borrow().clone()
    }

    fn set_first_name(&self, first_name: String) {
        if self.first_name() == first_name {
            return;
        }

        let priv_ = imp::User::from_instance(self);
        priv_.first_name.replace(first_name);
        self.notify("first-name");
    }

    pub fn last_name(&self) -> String {
        let priv_ = imp::User::from_instance(self);
        priv_.last_name.borrow().clone()
    }

    fn set_last_name(&self, last_name: String) {
        if self.last_name() == last_name {
            return;
        }

        let priv_ = imp::User::from_instance(self);
        priv_.last_name.replace(last_name);
        self.notify("last-name");
    }
}
