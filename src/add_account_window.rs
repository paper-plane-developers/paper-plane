use gtk::glib;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use adw::NavigationDirection;
    use glib::subclass;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/add_account_window.ui")]
    pub struct AddAccountWindow {
        #[template_child]
        pub content_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub phone_number_next: TemplateChild<gtk::Button>,
    }

    impl ObjectSubclass for AddAccountWindow {
        const NAME: &'static str = "AddAccountWindow";
        type Type = super::AddAccountWindow;
        type ParentType = adw::Window;
        type Interfaces = ();
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                content_leaflet: TemplateChild::default(),
                phone_number_next: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddAccountWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let leaflet = &*self.content_leaflet;
            self.phone_number_next
                .connect_clicked(glib::clone!(@weak leaflet => move |_| {
                    leaflet.navigate(NavigationDirection::Forward);
                }));
        }
    }

    impl WidgetImpl for AddAccountWindow {}
    impl WindowImpl for AddAccountWindow {}
    impl AdwWindowImpl for AddAccountWindow {}
}

glib::wrapper! {
    pub struct AddAccountWindow(ObjectSubclass<imp::AddAccountWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl AddAccountWindow {
    pub fn new() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create AddAccountWindow")
    }
}
