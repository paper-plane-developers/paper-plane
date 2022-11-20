use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::types::BasicGroup as TdBasicGroup;

use crate::tdlib::BoxedChatMemberStatus;

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
                    glib::ParamSpecInt64::builder("id").read_only().build(),
                    glib::ParamSpecInt::builder("member-count")
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
    pub(crate) fn from_td_object(td_basic_group: TdBasicGroup) -> Self {
        let basic_group: BasicGroup = glib::Object::builder().build();
        let imp = basic_group.imp();

        let status = BoxedChatMemberStatus(td_basic_group.status);

        imp.id.set(td_basic_group.id);
        imp.member_count.set(td_basic_group.member_count);
        imp.status.replace(Some(status));

        basic_group
    }

    pub(crate) fn update(&self, td_basic_group: TdBasicGroup) {
        self.set_member_count(td_basic_group.member_count);
        self.set_status(BoxedChatMemberStatus(td_basic_group.status));
    }

    pub(crate) fn id(&self) -> i64 {
        self.imp().id.get()
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
