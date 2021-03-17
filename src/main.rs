use gtk::gio;

mod add_account_window;
mod application;
mod chat_page;
mod config;
mod dialog_row;
mod message_row;
mod telegram;
mod window;

use application::TelegrandApplication;

fn main() {
    gtk::init().expect("Unable to initialize GTK");
    adw::init();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() +
        "/resources.gresource").expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = TelegrandApplication::new();
    app.run();
}
