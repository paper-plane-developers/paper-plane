use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use gtk::gio;
    use gtk::CompositeTemplate;

    use crate::config;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/preferences_window.ui")]
    pub struct PreferencesWindow {
        #[template_child]
        pub server_address_entry: TemplateChild<gtk::Entry>,

        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "PreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn new() -> Self {
            Self {
                server_address_entry: TemplateChild::default(),
                settings: gio::Settings::new(config::APP_ID),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
}

glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl PreferencesWindow {
    pub fn new() -> Self {
        let preferences_window = glib::Object::new(&[])
            .expect("Failed to create PreferencesWindow");

        let self_ = imp::PreferencesWindow::from_instance(&preferences_window);
        self_.settings
            .bind("custom-server-address", &*self_.server_address_entry, "text")
            .build();

        preferences_window
    }
}
