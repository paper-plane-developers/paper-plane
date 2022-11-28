use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::Update;
use tdlib::types::User as TdUser;

use crate::tdlib::{Avatar, BoxedUserStatus, BoxedUserType};
use crate::Session;

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct User {
        pub(super) id: Cell<i64>,
        pub(super) type_: RefCell<Option<BoxedUserType>>,
        pub(super) first_name: RefCell<String>,
        pub(super) last_name: RefCell<String>,
        pub(super) username: RefCell<String>,
        pub(super) phone_number: RefCell<String>,
        pub(super) avatar: RefCell<Option<Avatar>>,
        pub(super) status: RefCell<Option<BoxedUserStatus>>,
        pub(super) session: WeakRef<Session>,
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
                    glib::ParamSpecInt64::builder("id").read_only().build(),
                    glib::ParamSpecBoxed::builder::<BoxedUserType>("type")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("first-name")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("last-name")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("username")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("phone-number")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<Avatar>("avatar")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<BoxedUserStatus>("status")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<Session>("session")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "id" => obj.id().to_value(),
                "type" => obj.type_().to_value(),
                "first-name" => obj.first_name().to_value(),
                "last-name" => obj.last_name().to_value(),
                "username" => obj.username().to_value(),
                "phone-number" => obj.phone_number().to_value(),
                "avatar" => obj.avatar().to_value(),
                "status" => obj.status().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct User(ObjectSubclass<imp::User>);
}

impl User {
    pub(crate) fn from_td_object(td_user: TdUser, session: &Session) -> Self {
        let user: User = glib::Object::builder().build();
        let imp = user.imp();

        let type_ = BoxedUserType(td_user.r#type);
        let username = td_user
            .usernames
            .map(|u| u.editable_username)
            .unwrap_or_default();
        let avatar = td_user.profile_photo.map(Avatar::from);
        let status = BoxedUserStatus(td_user.status);

        imp.id.set(td_user.id);
        imp.type_.replace(Some(type_));
        imp.first_name.replace(td_user.first_name);
        imp.last_name.replace(td_user.last_name);
        imp.username.replace(username);
        imp.phone_number.replace(td_user.phone_number);
        imp.avatar.replace(avatar);
        imp.status.replace(Some(status));
        imp.session.set(Some(session));

        user
    }

    pub(crate) fn handle_update(&self, update: Update) {
        match update {
            Update::User(data) => {
                self.set_type(BoxedUserType(data.user.r#type));
                self.set_first_name(data.user.first_name);
                self.set_last_name(data.user.last_name);
                self.set_username(
                    data.user
                        .usernames
                        .map(|u| u.editable_username)
                        .unwrap_or_default(),
                );
                self.set_phone_number(data.user.phone_number);
                self.set_status(BoxedUserStatus(data.user.status));
                self.set_avatar(data.user.profile_photo.map(Into::into));
            }
            Update::UserStatus(data) => self.set_status(BoxedUserStatus(data.status)),
            _ => {}
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.imp().id.get()
    }

    pub(crate) fn type_(&self) -> BoxedUserType {
        self.imp().type_.borrow().as_ref().unwrap().to_owned()
    }

    fn set_type(&self, type_: BoxedUserType) {
        if self.imp().type_.borrow().as_ref() == Some(&type_) {
            return;
        }
        self.imp().type_.replace(Some(type_));
        self.notify("type");
    }

    pub(crate) fn first_name(&self) -> String {
        self.imp().first_name.borrow().to_owned()
    }

    fn set_first_name(&self, first_name: String) {
        if self.first_name() == first_name {
            return;
        }
        self.imp().first_name.replace(first_name);
        self.notify("first-name");
    }

    pub(crate) fn last_name(&self) -> String {
        self.imp().last_name.borrow().to_owned()
    }

    fn set_last_name(&self, last_name: String) {
        if self.last_name() == last_name {
            return;
        }
        self.imp().last_name.replace(last_name);
        self.notify("last-name");
    }

    pub(crate) fn username(&self) -> String {
        self.imp().username.borrow().to_owned()
    }

    fn set_username(&self, username: String) {
        if self.username() == username {
            return;
        }
        self.imp().username.replace(username);
        self.notify("username");
    }

    pub(crate) fn phone_number(&self) -> String {
        self.imp().phone_number.borrow().to_owned()
    }

    fn set_phone_number(&self, phone_number: String) {
        if self.phone_number() == phone_number {
            return;
        }
        self.imp().phone_number.replace(phone_number);
        self.notify("phone-number");
    }

    pub(crate) fn avatar(&self) -> Option<Avatar> {
        self.imp().avatar.borrow().to_owned()
    }

    fn set_avatar(&self, avatar: Option<Avatar>) {
        if self.avatar() == avatar {
            return;
        }
        self.imp().avatar.replace(avatar);
        self.notify("avatar");
    }

    pub(crate) fn connect_avatar_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("avatar"), f)
    }

    pub(crate) fn status(&self) -> BoxedUserStatus {
        self.imp().status.borrow().as_ref().unwrap().to_owned()
    }

    fn set_status(&self, status: BoxedUserStatus) {
        if self.imp().status.borrow().as_ref() == Some(&status) {
            return;
        }
        self.imp().status.replace(Some(status));
        self.notify("status");
    }

    pub(crate) fn session(&self) -> Session {
        self.imp().session.upgrade().unwrap()
    }
}
