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
    #[properties(wrapper_type = super::BasicGroup)]
    pub(crate) struct BasicGroup {
        #[property(get, set, construct_only)]
        pub(super) id: OnceCell<i64>,
        #[property(get)]
        pub(super) member_count: Cell<i32>,
        #[property(get)]
        pub(super) status: RefCell<model::BoxedChatMemberStatus>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BasicGroup {
        const NAME: &'static str = "BasicGroup";
        type Type = super::BasicGroup;
    }

    impl ObjectImpl for BasicGroup {
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
    pub(crate) struct BasicGroup(ObjectSubclass<imp::BasicGroup>);
}

impl From<tdlib::types::BasicGroup> for BasicGroup {
    fn from(td_basic_group: tdlib::types::BasicGroup) -> Self {
        let obj: Self = glib::Object::builder()
            .property("id", td_basic_group.id)
            .build();

        let imp = obj.imp();

        imp.member_count.set(td_basic_group.member_count);
        imp.status
            .replace(model::BoxedChatMemberStatus(td_basic_group.status));

        obj
    }
}

impl BasicGroup {
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

    pub(crate) fn update(&self, td_basic_group: tdlib::types::BasicGroup) {
        self.set_member_count(td_basic_group.member_count);
        self.set_status(model::BoxedChatMemberStatus(td_basic_group.status));
    }
}
