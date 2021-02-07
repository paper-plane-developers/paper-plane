use gtk::glib;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use glib::subclass;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/window.ui")]
    pub struct TelegrandWindow {
    }

    impl ObjectSubclass for TelegrandWindow {
        const NAME: &'static str = "TelegrandWindow";
        type Type = super::TelegrandWindow;
        type ParentType = adw::ApplicationWindow;
        type Interfaces = ();
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {}
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TelegrandWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for TelegrandWindow {}
    impl WindowImpl for TelegrandWindow {}
    impl ApplicationWindowImpl for TelegrandWindow {}
    impl AdwApplicationWindowImpl for TelegrandWindow {}
}

glib::wrapper! {
    pub struct TelegrandWindow(ObjectSubclass<imp::TelegrandWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl TelegrandWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P) -> Self {
        glib::Object::new(&[("application", app)])
            .expect("Failed to create TelegrandWindow")
    }
}
