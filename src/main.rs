mod application;
#[rustfmt::skip]
mod config;
mod login;
mod preferences_window;
mod proxy;
mod session;
mod utils;
mod window;

use self::application::Application;
use self::login::Login;
use self::preferences_window::PreferencesWindow;
use self::session::Session;
use self::window::Window;

use config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::{gettext, LocaleCategory};
use gtk::{
    gio, glib,
    prelude::{ApplicationExt, ApplicationExtManual, IsA},
};
use once_cell::sync::{Lazy, OnceCell};

use std::{path::PathBuf, str::FromStr};

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

pub static DATA_DIR: OnceCell<PathBuf> = OnceCell::new();

fn main() {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name("Telegrand");

    gtk::init().expect("Unable to start GTK4");
    adw::init();

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = setup_cli(Application::new());

    // Command line handling
    app.connect_handle_local_options(|_, dict| {
        if dict.contains("version") {
            // Print version ...
            println!("telegrand {}", config::VERSION);
            // ... and exit application.
            1
        } else {
            let log_level = match dict.lookup::<String>("log-level").unwrap() {
                Some(level) => log::Level::from_str(&level).expect("Error on parsing log-level"),
                // Standard log levels if not specified by user
                None => log::Level::Warn,
            };

            // TODO: Change to syslog when tdlib v1.8 is out where messages can be redirected.
            std::env::set_var("RUST_LOG", log_level.as_str());
            pretty_env_logger::init();

            DATA_DIR
                .set(
                    #[cfg(not(debug_assertions))]
                    default_data_dir(),
                    #[cfg(debug_assertions)]
                    dict.lookup::<String>("data-dir")
                        .unwrap()
                        .map(PathBuf::from)
                        .unwrap_or_else(default_data_dir),
                )
                .unwrap();

            -1
        }
    });

    app.run();
}

fn setup_cli<A: IsA<gio::Application>>(app: A) -> A {
    app.add_main_option(
        "version",
        b'v'.into(),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        &gettext("Prints application version"),
        None,
    );

    app.add_main_option(
        "log-level",
        b'l'.into(),
        glib::OptionFlags::NONE,
        glib::OptionArg::String,
        &gettext("Specify the minimum log level"),
        Some("error|warn|info|debug|trace"),
    );

    #[cfg(debug_assertions)]
    app.add_main_option(
        "data-dir",
        b'd'.into(),
        glib::OptionFlags::NONE,
        glib::OptionArg::String,
        &gettext("Specify a different data directory"),
        Some(&gettext("DIRECTORY")),
    );

    app
}

fn default_data_dir() -> PathBuf {
    // TODO: In the future when multi account is a thing, only use the parent directory.
    PathBuf::from(glib::user_data_dir().to_str().unwrap())
        .join("telegrand")
        .join("db0")
}
