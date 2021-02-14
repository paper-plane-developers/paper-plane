use gtk::prelude::*;
use gtk::glib;
use gtk::gio;
use std::env::args;

mod add_account_window;
mod config;
mod window;
mod telegram;

use window::TelegrandWindow;

fn main() {
    gtk::init().expect("Failed to initialize GTK");
    adw::init();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/resources.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

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
