use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::{Update, UserStatus, UserType};
use tdlib::types::User as TdUser;

use crate::session::Avatar;
use crate::Session;

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserType")]
pub struct BoxedUserType(pub UserType);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserStatus")]
pub struct BoxedUserStatus(pub UserStatus);

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct User {
        pub id: Cell<i64>,
        pub type_: RefCell<Option<BoxedUserType>>,
        pub first_name: RefCell<String>,
        pub last_name: RefCell<String>,
        pub username: RefCell<String>,
        pub phone_number: RefCell<String>,
        pub avatar: OnceCell<Avatar>,
        pub status: RefCell<Option<BoxedUserStatus>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for User {
        const NAME: &'static str = "User";
        type Type = super::User;
    }

    impl ObjectImpl for User {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::new(
                        "id",
                        "Id",
                        "The id of this user",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "type",
                        "Type",
                        "The type of this user",
                        BoxedUserType::static_type(),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "first-name",
                        "First Name",
                        "The first name of this user",
                        Some(""),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "last-name",
                        "Last Name",
                        "The last name of this user",
                        Some(""),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "username",
                        "Username",
                        "The username of this user",
                        Some(""),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "phone-number",
                        "Phone Number",
                        "The phone number of this user",
                        Some(""),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "avatar",
                        "Avatar",
                        "The avatar of this user",
                        Avatar::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "status",
                        "Status",
                        "The status of this user",
                        BoxedUserStatus::static_type(),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "type" => obj.set_type(value.get().unwrap()),
                "first-name" => {
                    obj.set_first_name(value.get::<Option<String>>().unwrap().unwrap_or_default())
                }
                "last-name" => {
                    obj.set_last_name(value.get::<Option<String>>().unwrap().unwrap_or_default())
                }
                "username" => {
                    obj.set_username(value.get::<Option<String>>().unwrap().unwrap_or_default())
                }
                "phone-number" => {
                    obj.set_phone_number(value.get::<Option<String>>().unwrap().unwrap_or_default())
                }
                "avatar" => self.avatar.set(value.get().unwrap()).unwrap(),
                "status" => obj.set_status(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "type" => obj.type_().to_value(),
                "first-name" => obj.first_name().to_value(),
                "last-name" => obj.last_name().to_value(),
                "username" => obj.username().to_value(),
                "phone-number" => obj.phone_number().to_value(),
                "avatar" => obj.avatar().to_value(),
                "status" => obj.status().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let avatar = obj.avatar();
            let user_expression = gtk::ConstantExpression::new(obj);
            super::User::full_name_expression(&user_expression).bind(
                avatar,
                "display-name",
                glib::Object::NONE,
            );
        }
    }
}

glib::wrapper! {
    pub struct User(ObjectSubclass<imp::User>);
}

impl User {
    pub fn from_td_object(user: TdUser, session: &Session) -> Self {
        let avatar = Avatar::new(session);
        avatar.update_from_user_photo(user.profile_photo);

        glib::Object::new(&[
            ("id", &user.id),
            ("type", &BoxedUserType(user.r#type)),
            ("first-name", &user.first_name),
            ("last-name", &user.last_name),
            ("username", &user.username),
            ("phone-number", &user.phone_number),
            ("status", &BoxedUserStatus(user.status)),
            ("avatar", &avatar),
        ])
        .expect("Failed to create User")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::User(data) => {
                self.set_type(BoxedUserType(data.user.r#type));
                self.set_first_name(data.user.first_name);
                self.set_last_name(data.user.last_name);
                self.set_username(data.user.username);
                self.set_phone_number(data.user.phone_number);
                self.set_status(BoxedUserStatus(data.user.status));
                self.avatar()
                    .update_from_user_photo(data.user.profile_photo);
            }
            Update::UserStatus(data) => self.set_status(BoxedUserStatus(data.status)),
            _ => {}
        }
    }

    pub fn id(&self) -> i64 {
        self.imp().id.get()
    }

    pub fn type_(&self) -> BoxedUserType {
        self.imp().type_.borrow().as_ref().unwrap().to_owned()
    }

    pub fn set_type(&self, type_: BoxedUserType) {
        if self.imp().type_.borrow().as_ref() == Some(&type_) {
            return;
        }
        self.imp().type_.replace(Some(type_));
        self.notify("type");
    }

    pub fn first_name(&self) -> String {
        self.imp().first_name.borrow().to_owned()
    }

    pub fn set_first_name(&self, first_name: String) {
        if self.first_name() == first_name {
            return;
        }
        self.imp().first_name.replace(first_name);
        self.notify("first-name");
    }

    pub fn last_name(&self) -> String {
        self.imp().last_name.borrow().to_owned()
    }

    pub fn set_last_name(&self, last_name: String) {
        if self.last_name() == last_name {
            return;
        }
        self.imp().last_name.replace(last_name);
        self.notify("last-name");
    }

    pub fn username(&self) -> String {
        self.imp().username.borrow().to_owned()
    }

    pub fn set_username(&self, username: String) {
        if self.username() == username {
            return;
        }
        self.imp().username.replace(username);
        self.notify("username");
    }

    pub fn phone_number(&self) -> String {
        self.imp().phone_number.borrow().to_owned()
    }

    pub fn set_phone_number(&self, phone_number: String) {
        if self.phone_number() == phone_number {
            return;
        }
        self.imp().phone_number.replace(phone_number);
        self.notify("phone-number");
    }

    pub fn avatar(&self) -> &Avatar {
        self.imp().avatar.get().unwrap()
    }

    pub fn status(&self) -> BoxedUserStatus {
        self.imp().status.borrow().as_ref().unwrap().to_owned()
    }

    pub fn set_status(&self, status: BoxedUserStatus) {
        if self.imp().status.borrow().as_ref() == Some(&status) {
            return;
        }
        self.imp().status.replace(Some(status));
        self.notify("status");
    }

    pub fn full_name_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        let first_name_expression =
            gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "first-name");
        let last_name_expression =
            gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "last-name");
        gtk::ClosureExpression::with_callback(
            &[first_name_expression, last_name_expression],
            |args| {
                let first_name = args[1].get::<String>().unwrap();
                let last_name = args[2].get::<String>().unwrap();
                format!("{} {}", first_name, last_name).trim().to_owned()
            },
        )
        .upcast()
    }
}
