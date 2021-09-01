use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-event-row.ui")]
    pub struct EventRow {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventRow {
        const NAME: &'static str = "ContentEventRow";
        type Type = super::EventRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EventRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_string(
                    "label",
                    "Label",
                    "The label for this event",
                    None,
                    glib::ParamFlags::READWRITE,
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
                "label" => obj.set_label(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
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
    pub struct EventRow(ObjectSubclass<imp::EventRow>)
        @extends gtk::Widget, adw::Bin;
}

impl EventRow {
    pub fn new(label: String) -> Self {
        glib::Object::new(&[("label", &label)]).expect("Failed to create EventRow")
    }

    pub fn label(&self) -> String {
        let self_ = imp::EventRow::from_instance(self);
        self_.label.text().to_string()
    }

    pub fn set_label(&self, label: &str) {
        let self_ = imp::EventRow::from_instance(self);
        self_.label.set_markup(label);
    }
}
