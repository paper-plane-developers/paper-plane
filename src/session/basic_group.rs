use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdgrand::enums::Update;
use tdgrand::types::BasicGroup as TdBasicGroup;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub struct BasicGroup {
        pub id: Cell<i64>,
        pub member_count: Cell<i32>,
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "member-count" => obj.member_count().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct BasicGroup(ObjectSubclass<imp::BasicGroup>);
}

impl BasicGroup {
    pub fn from_td_object(basic_group: &TdBasicGroup) -> Self {
        glib::Object::new(&[
            ("id", &basic_group.id),
            ("member-count", &basic_group.member_count),
        ])
        .expect("Failed to create BasicGroup")
    }

    pub fn handle_update(&self, update: &Update) {
        if let Update::BasicGroup(data) = update {
            self.set_member_count(data.basic_group.member_count);
        }
    }

    pub fn id(&self) -> i64 {
        self.imp().id.get()
    }

    pub fn member_count(&self) -> i32 {
        self.imp().member_count.get()
    }

    pub fn set_member_count(&self, member_count: i32) {
        if self.member_count() == member_count {
            return;
        }

        self.imp().member_count.set(member_count);
        self.notify("member-count");
    }
}
