use gtk::{glib, prelude::*, subclass::prelude::*};
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
        pub username: RefCell<String>,
        pub phone_number: RefCell<String>,
        pub avatar: OnceCell<Avatar>,
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
                    glib::ParamSpec::new_string(
                        "username",
                        "Username",
                        "The username of this user",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_string(
                        "phone-number",
                        "Phone Number",
                        "The phone number of this user",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "avatar",
                        "Avatar",
                        "The avatar of this chat",
                        Avatar::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "id" => self.id.set(value.get().unwrap()),
                "first-name" => obj.set_first_name(value.get().unwrap()),
                "last-name" => obj.set_last_name(value.get().unwrap()),
                "username" => obj.set_username(value.get().unwrap()),
                "phone-number" => obj.set_phone_number(value.get().unwrap()),
                "avatar" => self.avatar.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "first-name" => obj.first_name().to_value(),
                "last-name" => obj.last_name().to_value(),
                "username" => obj.username().to_value(),
                "phone-number" => obj.phone_number().to_value(),
                "avatar" => obj.avatar().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let avatar = obj.avatar();
            let user_expression = gtk::ConstantExpression::new(obj);
            let full_name_expression = super::User::full_name_expression(&user_expression);
            full_name_expression.bind(avatar, "display-name", gtk::NONE_WIDGET);
        }
    }
}

glib::wrapper! {
    pub struct User(ObjectSubclass<imp::User>);
}

impl User {
    pub fn new(id: i32, session: &Session) -> Self {
        let avatar = Avatar::new(session);
        glib::Object::new(&[("id", &id), ("avatar", &avatar)]).expect("Failed to create User")
    }

    pub fn handle_update(&self, update: Update) {
        if let Update::User(data) = update {
            self.set_first_name(data.user.first_name);
            self.set_last_name(data.user.last_name);
            self.set_username(data.user.username);
            self.set_phone_number(data.user.phone_number);

            self.avatar()
                .update_from_user_photo(data.user.profile_photo);
        }
    }

    pub fn id(&self) -> i32 {
        let self_ = imp::User::from_instance(self);
        self_.id.get()
    }

    pub fn first_name(&self) -> String {
        let self_ = imp::User::from_instance(self);
        self_.first_name.borrow().to_owned()
    }

    fn set_first_name(&self, first_name: String) {
        if self.first_name() == first_name {
            return;
        }

        let self_ = imp::User::from_instance(self);
        self_.first_name.replace(first_name);
        self.notify("first-name");
    }

    pub fn first_name_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "first-name")
            .upcast()
    }

    pub fn last_name(&self) -> String {
        let self_ = imp::User::from_instance(self);
        self_.last_name.borrow().to_owned()
    }

    fn set_last_name(&self, last_name: String) {
        if self.last_name() == last_name {
            return;
        }

        let self_ = imp::User::from_instance(self);
        self_.last_name.replace(last_name);
        self.notify("last-name");
    }

    pub fn last_name_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "last-name")
            .upcast()
    }

    pub fn username(&self) -> String {
        let self_ = imp::User::from_instance(self);
        self_.username.borrow().to_owned()
    }

    fn set_username(&self, username: String) {
        if self.username() == username {
            return;
        }

        let self_ = imp::User::from_instance(self);
        self_.username.replace(username);
        self.notify("username");
    }

    pub fn username_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "username")
            .upcast()
    }

    pub fn phone_number(&self) -> String {
        let self_ = imp::User::from_instance(self);
        self_.phone_number.borrow().to_owned()
    }

    fn set_phone_number(&self, phone_number: String) {
        if self.phone_number() == phone_number {
            return;
        }

        let self_ = imp::User::from_instance(self);
        self_.phone_number.replace(phone_number);
        self.notify("phone-number");
    }

    pub fn phone_number_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "phone-number")
            .upcast()
    }

    pub fn avatar(&self) -> &Avatar {
        let self_ = imp::User::from_instance(self);
        self_.avatar.get().unwrap()
    }

    pub fn full_name_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        let first_name_expression = User::first_name_expression(user_expression);
        let last_name_expression = User::last_name_expression(user_expression);

        gtk::ClosureExpression::new(
            move |args| -> String {
                let first_name = args[1].get::<&str>().unwrap();
                let last_name = args[2].get::<&str>().unwrap();
                format!("{} {}", first_name, last_name).trim().to_owned()
            },
            &[first_name_expression, last_name_expression],
        )
        .upcast()
    }
}
