use crate::config;
use crate::Window;
use gio::ApplicationFlags;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use log::{debug, info};

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::OnceCell;

    #[derive(Debug, Default)]
    pub struct Application {
        pub window: OnceCell<WeakRef<Window>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for Application {}

    impl gio::subclass::prelude::ApplicationImpl for Application {
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<Application>::activate");

            let priv_ = Application::from_instance(app);
            if let Some(window) = priv_.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            app.set_resource_base_path(Some("/com/github/melix99/telegrand/"));
            app.setup_css();

            let window = Window::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();

            app.get_main_window().present();
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<Application>::startup");
            self.parent_startup(app);
        }
    }

    impl GtkApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::empty()),
        ])
        .expect("Application initialization failed...")
    }

    fn get_main_window(&self) -> Window {
        let priv_ = imp::Application::from_instance(self);
        priv_.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(clone!(@weak self as app => move |_, _| {
            // This is needed to trigger the delete event and saving the window state
            app.get_main_window().close();
            app.quit();
        }));
        self.add_action(&action_quit);

        // About
        let action_about = gio::SimpleAction::new("about", None);
        action_about.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about_dialog();
        }));
        self.add_action(&action_about);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/com/github/melix99/telegrand/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialogBuilder::new()
            .program_name("Telegrand")
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/melix99/telegrand/")
            .version(config::VERSION)
            .transient_for(&self.get_main_window())
            .modal(true)
            .authors(vec!["Marco Melorio".into()])
            .artists(vec!["Marco Melorio".into()])
            .build();

        dialog.show();
    }

    pub fn run(&self) {
        info!("Telegrand ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
