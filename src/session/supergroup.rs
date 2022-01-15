use gtk::{glib, prelude::*, subclass::prelude::*};
use tdgrand::enums::Update;
use tdgrand::types::Supergroup as TdSupergroup;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub struct Supergroup {
        pub id: Cell<i64>,
        pub member_count: Cell<i32>,
        pub is_channel: Cell<bool>,
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
                    glib::ParamSpecInt64::new(
                        "id",
                        "Id",
                        "The id of this supergroup",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecInt::new(
                        "member-count",
                        "Member Count",
                        "The number of members of this supergroup",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-channel",
                        "Is Channel",
                        "Whether the supergroup is a channel or not",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "is-channel" => self.is_channel.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "member-count" => obj.member_count().to_value(),
                "is-channel" => obj.is_channel().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Supergroup(ObjectSubclass<imp::Supergroup>);
}

impl Supergroup {
    pub fn from_td_object(supergroup: &TdSupergroup) -> Self {
        glib::Object::new(&[
            ("id", &supergroup.id),
            ("member-count", &supergroup.member_count),
            ("is-channel", &supergroup.is_channel),
        ])
        .expect("Failed to create Supergroup")
    }

    pub fn handle_update(&self, update: &Update) {
        if let Update::Supergroup(data) = update {
            self.set_member_count(data.supergroup.member_count);
        }
    }

    pub fn id(&self) -> i64 {
        let self_ = imp::Supergroup::from_instance(self);
        self_.id.get()
    }

    pub fn member_count(&self) -> i32 {
        let self_ = imp::Supergroup::from_instance(self);
        self_.member_count.get()
    }

    pub fn set_member_count(&self, member_count: i32) {
        if self.member_count() == member_count {
            return;
        }

        let self_ = imp::Supergroup::from_instance(self);
        self_.member_count.set(member_count);
        self.notify("member-count");
    }

    pub fn is_channel(&self) -> bool {
        let self_ = imp::Supergroup::from_instance(self);
        self_.is_channel.get()
    }
}
