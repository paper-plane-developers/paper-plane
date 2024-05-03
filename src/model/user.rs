use std::cell::OnceCell;
use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;
use crate::types::UserId;

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::User)]
    pub(crate) struct User {
        #[property(get, set, construct_only)]
        pub(super) session: glib::WeakRef<model::ClientStateSession>,
        #[property(get, set, construct_only)]
        pub(super) id: OnceCell<UserId>,
        #[property(get)]
        pub(super) user_type: RefCell<model::BoxedUserType>,
        #[property(get)]
        pub(super) first_name: RefCell<String>,
        #[property(get)]
        pub(super) last_name: RefCell<String>,
        #[property(get)]
        pub(super) username: RefCell<String>,
        #[property(get)]
        pub(super) phone_number: RefCell<String>,
        #[property(get)]
        pub(super) avatar: RefCell<Option<model::Avatar>>,
        #[property(get)]
        pub(super) status: RefCell<model::BoxedUserStatus>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for User {
        const NAME: &'static str = "User";
        type Type = super::User;
    }

    impl ObjectImpl for User {
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
}

glib::wrapper! {
    pub(crate) struct User(ObjectSubclass<imp::User>);
}

impl User {
    pub(crate) fn new(session: &model::ClientStateSession, td_user: tdlib::types::User) -> Self {
        let obj: Self = glib::Object::builder()
            .property("session", session)
            .property("id", td_user.id)
            .build();

        let imp = obj.imp();

        imp.user_type.replace(model::BoxedUserType(td_user.r#type));
        imp.first_name.replace(td_user.first_name);
        imp.last_name.replace(td_user.last_name);
        imp.username.replace(
            td_user
                .usernames
                .map(|u| u.editable_username)
                .unwrap_or_default(),
        );
        imp.phone_number.replace(td_user.phone_number);
        imp.avatar
            .replace(td_user.profile_photo.map(model::Avatar::from));
        imp.status.replace(model::BoxedUserStatus(td_user.status));

        obj
    }

    pub(crate) fn session_(&self) -> model::ClientStateSession {
        self.session().unwrap()
    }

    fn set_type(&self, user_type: model::BoxedUserType) {
        if self.user_type() == user_type {
            return;
        }
        self.imp().user_type.replace(user_type);
        self.notify_user_type();
    }

    fn set_first_name(&self, first_name: String) {
        if self.first_name() == first_name {
            return;
        }
        self.imp().first_name.replace(first_name);
        self.notify_first_name();
    }

    fn set_last_name(&self, last_name: String) {
        if self.last_name() == last_name {
            return;
        }
        self.imp().last_name.replace(last_name);
        self.notify_last_name();
    }

    fn set_username(&self, username: String) {
        if self.username() == username {
            return;
        }
        self.imp().username.replace(username);
        self.notify_username();
    }

    fn set_phone_number(&self, phone_number: String) {
        if self.phone_number() == phone_number {
            return;
        }
        self.imp().phone_number.replace(phone_number);
        self.notify_phone_number();
    }

    fn set_avatar(&self, avatar: Option<model::Avatar>) {
        if self.avatar() == avatar {
            return;
        }
        self.imp().avatar.replace(avatar);
        self.notify_avatar();
    }

    fn set_status(&self, status: model::BoxedUserStatus) {
        if self.status() == status {
            return;
        }
        self.imp().status.replace(status);
        self.notify_status();
    }

    pub(crate) fn update(&self, td_user: tdlib::types::User) {
        self.set_type(model::BoxedUserType(td_user.r#type));
        self.set_first_name(td_user.first_name);
        self.set_last_name(td_user.last_name);
        self.set_username(
            td_user
                .usernames
                .map(|u| u.editable_username)
                .unwrap_or_default(),
        );
        self.set_phone_number(td_user.phone_number);
        self.set_status(model::BoxedUserStatus(td_user.status));
        self.set_avatar(td_user.profile_photo.map(Into::into));
    }

    pub(crate) fn update_status(&self, status: tdlib::enums::UserStatus) {
        self.set_status(model::BoxedUserStatus(status));
    }
}
