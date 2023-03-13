use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-event-row.ui")]
    pub(crate) struct MessageListViewEventRow {
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageListViewEventRow {
        const NAME: &'static str = "MessageListViewEventRow";
        type Type = super::MessageListViewEventRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageListViewEventRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecString::builder("label").build()]);
            PROPERTIES.as_ref()
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

    impl WidgetImpl for MessageListViewEventRow {}
    impl BinImpl for MessageListViewEventRow {}
}

glib::wrapper! {
    pub(crate) struct MessageListViewEventRow(ObjectSubclass<imp::MessageListViewEventRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for MessageListViewEventRow {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageListViewEventRow {
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
