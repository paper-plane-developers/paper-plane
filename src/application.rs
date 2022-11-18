use gettextrs::gettext;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use log::{debug, info};

use crate::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::{PreferencesWindow, Window};

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwApplicationImpl;
    use glib::WeakRef;
    use once_cell::sync::OnceCell;

    #[derive(Debug, Default)]
    pub(crate) struct Application {
        pub(super) window: OnceCell<WeakRef<Window>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            debug!("GtkApplication<Application>::activate");

            let obj = self.obj();

            if let Some(window) = self.window.get() {
                window.upgrade().unwrap().present();
                return;
            }

            let window = Window::new(&obj);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            obj.main_window().present();
        }

        fn startup(&self) {
            debug!("GtkApplication<Application>::startup");

            info!("Telegrand ({})", APP_ID);
            info!("Version: {} ({})", VERSION, PROFILE);
            info!("Datadir: {}", PKGDATADIR);

            self.parent_startup();

            let obj = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            obj.setup_gactions();
            obj.setup_accels();
            obj.load_color_scheme();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub(crate) struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub(crate) fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/com/github/melix99/telegrand/")
            .build()
    }

    fn main_window(&self) -> Window {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(clone!(@weak self as app => move |_, _| {
            // This is needed to trigger the delete event and saving the window state
            app.main_window().close();
            app.quit();
        }));
        self.add_action(&action_quit);

        // Preferences
        let action_preferences = gio::SimpleAction::new("preferences", None);
        action_preferences.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_preferences();
        }));
        self.add_action(&action_preferences);

        // About
        let action_about = gio::SimpleAction::new("about", None);
        action_about.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about_dialog();
        }));
        self.add_action(&action_about);

        // Select chat
        let action_select_chat =
            gio::SimpleAction::new("select-chat", Some(glib::VariantTy::new("(ix)").unwrap()));
        action_select_chat.connect_activate(clone!(@weak self as app => move |_, data| {
            let (client_id, chat_id) = data.unwrap().get().unwrap();
            app.main_window().select_chat(client_id, chat_id);
        }));
        self.add_action(&action_select_chat);

        // New login on production server
        let action_new_login_production_server =
            gio::SimpleAction::new("new-login-production-server", None);
        action_new_login_production_server.connect_activate(
            clone!(@weak self as app => move |_, _| {
                app.main_window().session_manager().add_new_session(false);
            }),
        );
        self.add_action(&action_new_login_production_server);

        // New login on test server
        let action_new_login_test_server = gio::SimpleAction::new("new-login-test-server", None);
        action_new_login_test_server.connect_activate(clone!(@weak self as app => move |_, _| {
            app.main_window().session_manager().add_new_session(true);
        }));
        self.add_action(&action_new_login_test_server);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
    }

    fn load_color_scheme(&self) {
        let style_manager = adw::StyleManager::default();
        let settings = gio::Settings::new(APP_ID);
        match settings.string("color-scheme").as_ref() {
            "light" => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
            "dark" => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
            _ => style_manager.set_color_scheme(adw::ColorScheme::PreferLight),
        }
    }

    fn show_preferences(&self) {
        let preferences = PreferencesWindow::new();
        preferences.set_transient_for(Some(&self.main_window()));
        preferences.present();
    }

    fn show_about_dialog(&self) {
        let about = adw::AboutWindow::builder()
            .transient_for(&self.main_window())
            .application_name("Telegrand")
            .application_icon(APP_ID)
            .version(VERSION)
            .website("https://github.com/melix99/telegrand")
            .issue_url("https://github.com/melix99/telegrand/issues")
            .support_url("https://t.me/telegrandchat")
            .developer_name(&gettext("Telegrand developers"))
            .copyright("© 2021–2022 Marco Melorio")
            .license_type(gtk::License::Gpl30)
            .developers(vec![
                "Marco Melorio https://github.com/melix99".into(),
                "Marcus Behrendt https://github.com/marhkb".into(),
                "Carlod https://github.com/just-carlod".into(),
            ])
            .designers(vec!["Marco Melorio https://github.com/melix99".into()])
            .artists(vec![
                "Mateus Santos https://github.com/swyknox".into(),
                "noëlle https://github.com/jannuary".into(),
            ])
            .build();

        about.add_acknowledgement_section(
            Some(&gettext("Sponsors")),
            &["Alisson Lauffer", "Jordan Maris"],
        );

        about.present();
    }
}
