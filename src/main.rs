mod application;
#[rustfmt::skip]
mod config;
mod login;
mod utils;
mod window;

use self::application::Application;
use self::login::Login;
use self::window::Window;

use adw;
use config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::*;
use gtk::gio;
use once_cell::sync::Lazy;
use tokio;

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

fn main() {
    // Initialize logger, debug is carried out via debug!, info!, and warn!.
    pretty_env_logger::init();

    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)
        .expect("Unable to bind the text domain");
    textdomain(GETTEXT_PACKAGE)
        .expect("Unable to switch to the text domain");

    gtk::glib::set_application_name("Telegrand");
    gtk::glib::set_prgname(Some("telegrand"));

    gtk::init().expect("Unable to start GTK4");
    adw::init();

    let res = gio::Resource::load(RESOURCES_FILE)
        .expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = Application::new();
    app.run();
}
