use chrono::prelude::*;
use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*};
use tdgrand::enums::{Update, UserStatus, UserType};

use crate::session::Avatar;
use crate::Session;

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedUserType")]
pub struct BoxedUserType(pub UserType);

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedUserStatus")]
pub struct BoxedUserStatus(pub UserStatus);

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct User {
        pub id: Cell<i64>,
        pub type_: RefCell<BoxedUserType>,
        pub first_name: RefCell<String>,
        pub last_name: RefCell<String>,
        pub username: RefCell<String>,
        pub phone_number: RefCell<String>,
        pub avatar: OnceCell<Avatar>,
        pub status: RefCell<BoxedUserStatus>,
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
                    glib::ParamSpec::new_int64(
                        "id",
                        "Id",
                        "The id of this user",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "type",
                        "Type",
                        "The type of this user",
                        BoxedUserType::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                    glib::ParamSpec::new_boxed(
                        "status",
                        "Status",
                        "The status of this user",
                        BoxedUserStatus::static_type(),
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
                "id" => self.id.set(value.get().unwrap()),
                "type" => {
                    self.type_.replace(value.get().unwrap());
                }
                "first-name" => obj.set_first_name(value.get().unwrap()),
                "last-name" => obj.set_last_name(value.get().unwrap()),
                "username" => obj.set_username(value.get().unwrap()),
                "phone-number" => obj.set_phone_number(value.get().unwrap()),
                "avatar" => self.avatar.set(value.get().unwrap()).unwrap(),
                "status" => {
                    self.status.replace(value.get().unwrap());
                }
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
            let display_name_expression = super::User::display_name_expression(&user_expression);
            display_name_expression.bind(avatar, "display-name", gtk::NONE_WIDGET);
        }
    }
    pub fn time_ago(unix: i32) -> String {
        let current_time = Utc::now().timestamp();
        let unix: i64 = unix.into();
        let time_diff = current_time - unix;
        let minutes = time_diff / 60;
        let hours = minutes / 60;
        let days = hours / 24;
        let last_seen =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(unix.into(), 0), Utc);
        if days == 0 {
            if hours == 0 {
                if minutes <= 1 {
                    gettext("just now")
                } else {
                    gettext!("{} minutes ago", minutes)
                }
            } else if hours == 1 {
                gettext("an hour ago")
            } else {
                format!("{} hours ago", hours)
            }
        } else if days == 1 {
            gettext!("yesterday at {}", last_seen.format("%H:%M"))
        } else {
            gettext!(
                "{} at {}",
                last_seen.format("%d:%m:%Y"),
                last_seen.format("%H:%M")
            )
        }
    }
}

glib::wrapper! {
    pub struct User(ObjectSubclass<imp::User>);
}

impl User {
    pub fn new(id: i64, session: &Session) -> Self {
        let avatar = Avatar::new(session);
        glib::Object::new(&[("id", &id), ("avatar", &avatar)]).expect("Failed to create User")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::User(data) => {
                self.set_type(data.user.r#type);
                self.set_first_name(data.user.first_name);
                self.set_last_name(data.user.last_name);
                self.set_username(data.user.username);
                self.set_phone_number(data.user.phone_number);
                self.set_status(data.user.status);

                self.avatar()
                    .update_from_user_photo(data.user.profile_photo);
            }
            Update::UserStatus(data) => self.set_status(data.status),
            _ => {}
        }
    }

    pub fn id(&self) -> i64 {
        let self_ = imp::User::from_instance(self);
        self_.id.get()
    }

    pub fn type_(&self) -> BoxedUserType {
        let self_ = imp::User::from_instance(self);
        self_.type_.borrow().clone()
    }

    pub fn set_type(&self, type_: UserType) {
        if self.type_().0 == type_ {
            return;
        }
        let self_ = imp::User::from_instance(self);
        self_.type_.replace(BoxedUserType(type_));
        self.notify("type");
    }

    pub fn type_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "type").upcast()
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
    pub fn display_name_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        let full_name_expression = User::full_name_expression(&user_expression);
        let type_expression = User::type_expression(&user_expression);
        gtk::ClosureExpression::new(
            move |args| -> String {
                let mut display_name = args[1].get::<&str>().unwrap();
                let type_ = args[2].get::<BoxedUserType>().unwrap().0;
                if type_ == UserType::Deleted {
                    display_name = "Deleted Account"
                }
                String::from(display_name)
            },
            &[full_name_expression, type_expression],
        )
        .upcast()
    }

    fn status(&self) -> BoxedUserStatus {
        let self_ = imp::User::from_instance(self);
        self_.status.borrow().clone()
    }
    fn set_status(&self, status: UserStatus) {
        if self.status().0 == status {
            return;
        }
        let self_ = imp::User::from_instance(self);
        self_.status.replace(BoxedUserStatus(status));
        self.notify("status");
    }

    pub fn status_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "status").upcast()
    }
    pub fn formated_status_expression(user_expression: &gtk::Expression) -> gtk::Expression {
        let type_expression =
            gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "type_")
                .upcast();
        let status_expression =
            gtk::PropertyExpression::new(User::static_type(), Some(user_expression), "status")
                .upcast();
        gtk::ClosureExpression::new(
            |args| -> String {
                let user_status = &args[1].get::<BoxedUserStatus>().unwrap().0;
                let type_ = &args[2].get::<BoxedUserType>().unwrap().0;
                match type_ {
                    UserType::Regular => match user_status {
                        UserStatus::Online(_data) => gettext("online"),
                        UserStatus::Offline(data) => {
                            let unix = data.was_online;
                            format!(
                                "{} {}",
                                gettext("last seen"),
                                super::user::imp::time_ago(unix)
                            )
                        }
                        UserStatus::Recently => gettext("last seen recently"),
                        UserStatus::LastWeek => gettext("last seen last week"),
                        UserStatus::LastMonth => gettext("last seen last month"),
                        UserStatus::Empty => gettext("last seen a long time ago"),
                    },
                    UserType::Bot(_data) => gettext("bot"),
                    UserType::Deleted => gettext("last seen a long time ago"),
                    _ => String::new(),
                }
            },
            &[status_expression.clone(), type_expression],
        )
        .upcast()
    }
}
