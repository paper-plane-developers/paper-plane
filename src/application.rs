use crate::config;
use crate::window::ExampleApplicationWindow;
use gio::ApplicationFlags;
use glib::clone;
use glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use gtk_macros::action;
use log::{debug, info};
use once_cell::sync::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ExampleApplication {
        pub window: OnceCell<WeakRef<ExampleApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExampleApplication {
        const NAME: &'static str = "ExampleApplication";
        type Type = super::ExampleApplication;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for ExampleApplication {}

    impl gio::subclass::prelude::ApplicationImpl for ExampleApplication {
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<ExampleApplication>::activate");

            let priv_ = ExampleApplication::from_instance(app);
            if let Some(window) = priv_.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            app.set_resource_base_path(Some("/com/github/melix99/telegrand/"));
            app.setup_css();

            let window = ExampleApplicationWindow::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();

            app.get_main_window().present();
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<ExampleApplication>::startup");
            self.parent_startup(app);
        }
    }

    impl GtkApplicationImpl for ExampleApplication {}
}

glib::wrapper! {
    pub struct ExampleApplication(ObjectSubclass<imp::ExampleApplication>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl ExampleApplication {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::empty()),
        ])
        .expect("Application initialization failed...")
    }

    fn get_main_window(&self) -> ExampleApplicationWindow {
        let priv_ = imp::ExampleApplication::from_instance(self);
        priv_.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        action!(
            self,
            "quit",
            clone!(@weak self as app => move |_, _| {
                // This is needed to trigger the delete event
                // and saving the window state
                app.get_main_window().close();
                app.quit();
            })
        );

        // About
        action!(
            self,
            "about",
            clone!(@weak self as app => move |_, _| {
                app.show_about_dialog();
            })
        );
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
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
            .license_type(gtk::License::MitX11)
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
