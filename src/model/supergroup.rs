use std::cell::Cell;
use std::cell::OnceCell;
use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::Supergroup)]
    pub(crate) struct Supergroup {
        #[property(get, set, construct_only)]
        pub(super) id: OnceCell<i64>,
        #[property(get, set, construct_only)]
        pub(super) is_channel: OnceCell<bool>,
        #[property(get)]
        pub(super) username: RefCell<String>,
        #[property(get)]
        pub(super) member_count: Cell<i32>,
        #[property(get)]
        pub(super) status: RefCell<model::BoxedChatMemberStatus>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Supergroup {
        const NAME: &'static str = "Supergroup";
        type Type = super::Supergroup;
    }

    impl ObjectImpl for Supergroup {
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
    pub(crate) struct Supergroup(ObjectSubclass<imp::Supergroup>);
}

impl From<tdlib::types::Supergroup> for Supergroup {
    fn from(td_supergroup: tdlib::types::Supergroup) -> Self {
        let obj: Self = glib::Object::builder()
            .property("id", td_supergroup.id)
            .property("is-channel", td_supergroup.is_channel)
            .build();

        let imp = obj.imp();

        imp.username.replace(
            td_supergroup
                .usernames
                .map(|u| u.editable_username)
                .unwrap_or_default(),
        );
        imp.member_count.set(td_supergroup.member_count);
        imp.status
            .replace(model::BoxedChatMemberStatus(td_supergroup.status));

        obj
    }
}

impl Supergroup {
    fn set_username(&self, username: String) {
        if self.username() == username {
            return;
        }
        self.imp().username.replace(username);
        self.notify_username();
    }

    fn set_member_count(&self, member_count: i32) {
        if self.member_count() == member_count {
            return;
        }
        self.imp().member_count.set(member_count);
        self.notify_member_count();
    }

    fn set_status(&self, status: model::BoxedChatMemberStatus) {
        if self.status() == status {
            return;
        }
        self.imp().status.replace(status);
        self.notify_status();
    }

    pub(crate) fn update(&self, td_supergroup: tdlib::types::Supergroup) {
        self.set_username(
            td_supergroup
                .usernames
                .map(|u| u.editable_username)
                .unwrap_or_default(),
        );
        self.set_member_count(td_supergroup.member_count);
        self.set_status(model::BoxedChatMemberStatus(td_supergroup.status));
    }
}
