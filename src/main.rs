use gtk::prelude::*;
use gtk::gio;
use std::env::args;

mod add_account_window;
mod config;
mod window;

use add_account_window::AddAccountWindow;
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
        let win = TelegrandWindow::new(app);
        win.show();

        let add_account_window = AddAccountWindow::new();
        add_account_window.set_transient_for(Some(&win));
        add_account_window.show();
    });

    application.run(&args().collect::<Vec<_>>());
}
