use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::types::Supergroup as TdSupergroup;

use crate::tdlib::BoxedChatMemberStatus;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct Supergroup {
        pub(super) id: Cell<i64>,
        pub(super) username: RefCell<String>,
        pub(super) member_count: Cell<i32>,
        pub(super) is_channel: Cell<bool>,
        pub(super) status: RefCell<Option<BoxedChatMemberStatus>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Supergroup {
        const NAME: &'static str = "Supergroup";
        type Type = super::Supergroup;
    }

    impl ObjectImpl for Supergroup {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("id").read_only().build(),
                    glib::ParamSpecString::builder("username")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt::builder("member-count")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-channel")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<BoxedChatMemberStatus>("status")
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
                "username" => obj.username().to_value(),
                "member-count" => obj.member_count().to_value(),
                "is-channel" => obj.is_channel().to_value(),
                "status" => obj.status().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Supergroup(ObjectSubclass<imp::Supergroup>);
}

impl Supergroup {
    pub(crate) fn from_td_object(td_supergroup: TdSupergroup) -> Self {
        let supergroup: Supergroup = glib::Object::builder().build();
        let imp = supergroup.imp();

        let username = td_supergroup
            .usernames
            .map(|u| u.editable_username)
            .unwrap_or_default();
        let status = BoxedChatMemberStatus(td_supergroup.status);

        imp.id.set(td_supergroup.id);
        imp.username.replace(username);
        imp.member_count.set(td_supergroup.member_count);
        imp.is_channel.set(td_supergroup.is_channel);
        imp.status.replace(Some(status));

        supergroup
    }

    pub(crate) fn update(&self, td_supergroup: TdSupergroup) {
        self.set_username(
            td_supergroup
                .usernames
                .map(|u| u.editable_username)
                .unwrap_or_default(),
        );
        self.set_member_count(td_supergroup.member_count);
        self.set_status(BoxedChatMemberStatus(td_supergroup.status));
    }

    pub(crate) fn id(&self) -> i64 {
        self.imp().id.get()
    }

    pub(crate) fn username(&self) -> String {
        self.imp().username.borrow().clone()
    }

    fn set_username(&self, username: String) {
        if self.username() == username {
            return;
        }
        self.imp().username.replace(username);
        self.notify("username");
    }

    pub(crate) fn member_count(&self) -> i32 {
        self.imp().member_count.get()
    }

    fn set_member_count(&self, member_count: i32) {
        if self.member_count() == member_count {
            return;
        }
        self.imp().member_count.set(member_count);
        self.notify("member-count");
    }

    pub(crate) fn is_channel(&self) -> bool {
        self.imp().is_channel.get()
    }

    pub(crate) fn status(&self) -> BoxedChatMemberStatus {
        self.imp().status.borrow().to_owned().unwrap()
    }

    fn set_status(&self, status: BoxedChatMemberStatus) {
        if self.imp().status.borrow().as_ref() == Some(&status) {
            return;
        }
        self.imp().status.replace(Some(status));
        self.notify("status");
    }
}
