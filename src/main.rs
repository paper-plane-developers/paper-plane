mod application;
#[rustfmt::skip]
mod config;
mod login;
mod preferences_window;
mod session;
mod utils;
mod window;

use self::application::Application;
use self::login::Login;
use self::preferences_window::PreferencesWindow;
use self::session::Session;
use self::window::Window;

use config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::LocaleCategory;
use gtk::{gio, glib};
use once_cell::sync::Lazy;

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

fn main() {
    // Initialize logger
    pretty_env_logger::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name("Telegrand");

    gtk::init().expect("Unable to start GTK4");
    adw::init();

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = Application::new();
    app.run();
}
