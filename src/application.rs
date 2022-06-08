use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use log::{debug, info};
use tdlib::functions;

use crate::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::utils::spawn;
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
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<Application>::activate");

            if let Some(window) = self.window.get() {
                window.upgrade().unwrap().present();
                return;
            }

            let window = Window::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<Application>::startup");

            info!("Telegrand ({})", APP_ID);
            info!("Version: {} ({})", VERSION, PROFILE);
            info!("Datadir: {}", PKGDATADIR);

            self.parent_startup(app);

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_gactions();
            app.setup_accels();
            app.load_color_scheme();
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
        glib::Object::new(&[
            ("application-id", &Some(APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            (
                "resource-base-path",
                &Some("/com/github/melix99/telegrand/"),
            ),
        ])
        .expect("Application initialization failed...")
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

        // Mark message as viewed
        let action_mark_message_as_viewed = gio::SimpleAction::new(
            "mark-message-as-viewed",
            Some(glib::VariantTy::new("(ixx)").unwrap()),
        );
        action_mark_message_as_viewed.connect_activate(
            clone!(@weak self as app => move |_, data| {
                let (client_id, chat_id, message_id) = data.unwrap().get().unwrap();
                spawn(async move {
                    functions::view_messages(chat_id, 0, vec![message_id], true, client_id)
                        .await
                        .unwrap();
                });
            }),
        );
        self.add_action(&action_mark_message_as_viewed);

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
        let dialog = gtk::AboutDialog::builder()
            .program_name("Telegrand")
            .logo_icon_name(APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/melix99/telegrand/")
            .version(VERSION)
            .transient_for(&self.main_window())
            .modal(true)
            .authors(vec!["Marco Melorio".into(), "Marcus Behrendt".into()])
            .artists(vec![
                "Marco Melorio".into(),
                "Mateus Santos".into(),
                "noëlle".into(),
            ])
            .build();

        dialog.present();
    }
}
