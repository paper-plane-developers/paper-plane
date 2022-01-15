use gtk::glib;
use gtk::{prelude::*, subclass::prelude::*};

mod imp {
    use super::*;

    use glib::subclass::InitializingObject;
    use gtk::glib::clone;
    use gtk::{self, CompositeTemplate};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/add-account-row.ui")]
    pub struct AddAccountRow {
        #[template_child]
        pub image: TemplateChild<gtk::Image>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub menu: TemplateChild<gtk::PopoverMenu>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddAccountRow {
        const NAME: &'static str = "AddAccountRow";
        type Type = super::AddAccountRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddAccountRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let long_press_events = gtk::GestureLongPress::builder().delay_factor(2.0).build();
            long_press_events.connect_pressed(clone!(@weak obj => move |_, _, _| {
                Self::from_instance(&obj).menu.show();
            }));
            // A cancelled long press event is used to emulate a normal "click" event.
            long_press_events.connect_cancelled(clone!(@weak obj => move |_| {
                obj.activate_action("app.new-login-production-server", None).unwrap();
            }));

            obj.add_controller(&long_press_events);
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.image.unparent();
            self.label.unparent();
            self.menu.unparent();
        }
    }
    impl WidgetImpl for AddAccountRow {}
}

glib::wrapper! {
    pub struct AddAccountRow(ObjectSubclass<imp::AddAccountRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible;
}

impl AddAccountRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AddAccountRow")
    }
}

impl Default for AddAccountRow {
    fn default() -> Self {
        Self::new()
    }
}
