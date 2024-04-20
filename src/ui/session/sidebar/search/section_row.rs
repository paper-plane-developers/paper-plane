use std::cell::Cell;
use std::cell::RefCell;
use std::sync::OnceLock;

use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::ui;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/search/section_row.ui")]
    pub(crate) struct SectionRow {
        pub(super) section_type: Cell<ui::SidebarSearchSectionType>,
        pub(super) suffix: RefCell<Option<gtk::Widget>>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Inscription>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SectionRow {
        const NAME: &'static str = "PaplSidebarSearchSectionRow";
        type Type = super::SectionRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SectionRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecEnum::builder::<ui::SidebarSearchSectionType>("section-type")
                        .explicit_notify()
                        .build(),
                ]
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

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().update_content();
        }

        fn dispose(&self) {
            self.label.unparent();
            if let Some(suffix) = self.suffix.take() {
                suffix.unparent();
            }
        }
    }

    impl WidgetImpl for SectionRow {}
}

glib::wrapper! {
    pub(crate) struct SectionRow(ObjectSubclass<imp::SectionRow>)
        @extends gtk::Widget;
}

impl SectionRow {
    pub(crate) fn new(section_type: ui::SidebarSearchSectionType) -> Self {
        glib::Object::builder()
            .property("section-type", section_type)
            .build()
    }

    pub(crate) fn section_type(&self) -> ui::SidebarSearchSectionType {
        self.imp().section_type.get()
    }

    pub(crate) fn set_section_type(&self, section_type: ui::SidebarSearchSectionType) {
        if self.section_type() == section_type {
            return;
        }

        self.imp().section_type.set(section_type);

        self.update_content();

        self.notify("section-type");
    }

    fn update_content(&self) {
        let imp = self.imp();

        if let Some(suffix) = imp.suffix.take() {
            suffix.unparent();
        }

        match self.section_type() {
            ui::SidebarSearchSectionType::Chats => {
                imp.label.set_text(Some(&gettext("Chats")));
            }
            ui::SidebarSearchSectionType::Global => {
                imp.label.set_text(Some(&gettext("Global Search")));
            }
            ui::SidebarSearchSectionType::Recent => {
                imp.label.set_text(Some(&gettext("Recent")));

                let button = gtk::Button::builder()
                    .icon_name("clear-symbolic")
                    .action_name("sidebar-search.clear-recent-chats")
                    .build();
                button.add_css_class("flat");
                button.insert_before(self, gtk::Widget::NONE);
                imp.suffix.replace(Some(button.upcast()));
            }
        }
    }
}
