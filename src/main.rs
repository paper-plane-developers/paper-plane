use gtk::prelude::*;
use gtk::gdk;
use gtk::glib;
use gtk::gio;
use std::env::args;

mod add_account_window;
mod config;
mod window;
mod telegram;

use window::TelegrandWindow;

fn setup_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/com/github/melix99/telegrand/style.css");
    if let Some(display) = gdk::Display::get_default() {
        gtk::StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn main() {
    gtk::init().expect("Failed to initialize GTK");
    adw::init();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/resources.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    setup_css();

    let application = gtk::Application::new(
        Some("com.github.melix99.telegrand"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let win = TelegrandWindow::new(app, receiver);
        win.show();

        telegram::spawn(sender);
    });

    application.run(&args().collect::<Vec<_>>());
}
