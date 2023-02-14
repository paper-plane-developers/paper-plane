use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    use glib::subclass::InitializingObject;
    use gtk::glib::clone;
    use gtk::{self, CompositeTemplate};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/add-account-row.ui")]
    pub(crate) struct AddAccountRow {
        #[template_child]
        pub(super) image: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) menu: TemplateChild<gtk::PopoverMenu>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddAccountRow {
        const NAME: &'static str = "AddAccountRow";
        type Type = super::AddAccountRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddAccountRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let long_press_events = gtk::GestureLongPress::builder().delay_factor(2.0).build();
            long_press_events.connect_pressed(clone!(@weak obj => move |_, _, _| {
                obj.imp().menu.popup();
            }));
            // A cancelled long press event is used to emulate a normal "click" event.
            long_press_events.connect_cancelled(clone!(@weak obj => move |_| {
                obj.activate_action("app.new-login-production-server", None).unwrap();
            }));

            obj.add_controller(long_press_events);
        }

        fn dispose(&self) {
            self.image.unparent();
            self.label.unparent();
            self.menu.unparent();
        }
    }
    impl WidgetImpl for AddAccountRow {}
}

glib::wrapper! {
    pub(crate) struct AddAccountRow(ObjectSubclass<imp::AddAccountRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible;
}

impl AddAccountRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for AddAccountRow {
    fn default() -> Self {
        Self::new()
    }
}
