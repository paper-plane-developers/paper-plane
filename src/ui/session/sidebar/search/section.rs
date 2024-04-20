use std::cell::Cell;
use std::sync::OnceLock;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "SidebarSearchSectionType")]
pub(crate) enum SectionType {
    #[default]
    Chats,
    Global,
    Recent,
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Section {
        pub(super) section_type: Cell<SectionType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Section {
        const NAME: &'static str = "PaplSidebarSearchSection";
        type Type = super::Section;
    }

    impl ObjectImpl for Section {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecEnum::builder::<SectionType>("section-type")
                    .explicit_notify()
                    .build()]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "section-type" => obj.set_section_type(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "section-type" => obj.section_type().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Section(ObjectSubclass<imp::Section>);
}

impl Section {
    pub(crate) fn new(section_type: SectionType) -> Self {
        glib::Object::builder()
            .property("section-type", section_type)
            .build()
    }

    pub(crate) fn section_type(&self) -> SectionType {
        self.imp().section_type.get()
    }

    pub(crate) fn set_section_type(&self, section_type: SectionType) {
        if self.section_type() == section_type {
            return;
        }
        self.imp().section_type.set(section_type);
        self.notify("section-type");
    }
}
