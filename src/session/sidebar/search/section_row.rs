use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::session::sidebar::search::SectionType;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="SidebarSearchSectionRow" parent="GtkWidget">
        <child>
          <object class="GtkLabel" id="label">
            <property name="ellipsize">end</property>
            <style>
              <class name="heading"/>
            </style>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    pub(crate) struct SectionRow {
        pub(super) section_type: Cell<SectionType>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SectionRow {
        const NAME: &'static str = "SidebarSearchSectionRow";
        type Type = super::SectionRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SectionRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecEnum::new(
                    "section-type",
                    "Section Type",
                    "The type of the section",
                    SectionType::static_type(),
                    SectionType::default() as i32,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                "section-type" => obj.set_section_type(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "section-type" => obj.section_type().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.update_content();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.label.unparent();
        }
    }

    impl WidgetImpl for SectionRow {}
}

glib::wrapper! {
    pub(crate) struct SectionRow(ObjectSubclass<imp::SectionRow>)
        @extends gtk::Widget;
}

impl SectionRow {
    pub(crate) fn new(section_type: SectionType) -> Self {
        glib::Object::new(&[("section-type", &section_type)])
            .expect("Failed to create SidebarSearchSectionRow")
    }

    pub(crate) fn section_type(&self) -> SectionType {
        self.imp().section_type.get()
    }

    pub(crate) fn set_section_type(&self, section_type: SectionType) {
        if self.section_type() == section_type {
            return;
        }

        self.imp().section_type.set(section_type);

        self.update_content();

        self.notify("section-type");
    }

    fn update_content(&self) {
        let imp = self.imp();

        match self.section_type() {
            SectionType::Chats => {
                imp.label.set_label(&gettext("Chats"));
            }
            SectionType::Recent => {
                imp.label.set_label(&gettext("Recent"));
            }
        }
    }
}
