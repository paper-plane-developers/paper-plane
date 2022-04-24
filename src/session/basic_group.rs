use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::Update;
use tdlib::types::BasicGroup as TdBasicGroup;

use crate::session::chat::BoxedChatMemberStatus;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct BasicGroup {
        pub(super) id: Cell<i64>,
        pub(super) member_count: Cell<i32>,
        pub(super) status: RefCell<Option<BoxedChatMemberStatus>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BasicGroup {
        const NAME: &'static str = "BasicGroup";
        type Type = super::BasicGroup;
    }

    impl ObjectImpl for BasicGroup {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::new(
                        "id",
                        "Id",
                        "The id of this basic group",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecInt::new(
                        "member-count",
                        "Member Count",
                        "The number of members of this basic group",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "status",
                        "Status",
                        "Own user status in this basic group",
                        BoxedChatMemberStatus::static_type(),
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
                "member-count" => obj.set_member_count(value.get().unwrap()),
                "status" => obj.set_status(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "member-count" => obj.member_count().to_value(),
                "status" => obj.status().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct BasicGroup(ObjectSubclass<imp::BasicGroup>);
}

impl BasicGroup {
    pub(crate) fn from_td_object(basic_group: &TdBasicGroup) -> Self {
        glib::Object::new(&[
            ("id", &basic_group.id),
            ("member-count", &basic_group.member_count),
            ("status", &BoxedChatMemberStatus(basic_group.status.clone())),
        ])
        .expect("Failed to create BasicGroup")
    }

    pub(crate) fn handle_update(&self, update: &Update) {
        if let Update::BasicGroup(data) = update {
            self.set_member_count(data.basic_group.member_count);
            self.set_status(BoxedChatMemberStatus(data.basic_group.status.clone()));
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.imp().id.get()
    }

    pub(crate) fn member_count(&self) -> i32 {
        self.imp().member_count.get()
    }

    pub(crate) fn set_member_count(&self, member_count: i32) {
        if self.member_count() == member_count {
            return;
        }

        self.imp().member_count.set(member_count);
        self.notify("member-count");
    }

    pub(crate) fn status(&self) -> BoxedChatMemberStatus {
        self.imp().status.borrow().to_owned().unwrap()
    }

    pub(crate) fn set_status(&self, status: BoxedChatMemberStatus) {
        if self.imp().status.borrow().as_ref() == Some(&status) {
            return;
        }
        self.imp().status.replace(Some(status));
        self.notify("status");
    }
}
