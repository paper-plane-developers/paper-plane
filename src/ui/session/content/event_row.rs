use std::sync::OnceLock;

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/event_row.ui")]
    pub(crate) struct EventRow {
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventRow {
        const NAME: &'static str = "PaplEventRow";
        type Type = super::EventRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EventRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| vec![glib::ParamSpecString::builder("label").build()])
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "label" => obj.set_label(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "label" => obj.label().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for EventRow {}
    impl BinImpl for EventRow {}
}

glib::wrapper! {
    pub(crate) struct EventRow(ObjectSubclass<imp::EventRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for EventRow {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn label(&self) -> String {
        self.imp().label.text().to_string()
    }

    pub(crate) fn set_label(&self, label: &str) {
        self.imp().label.set_label(label);
    }
}
