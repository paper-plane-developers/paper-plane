use glib::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ClientStateLoggingOut;

    #[glib::object_subclass]
    impl ObjectSubclass for ClientStateLoggingOut {
        const NAME: &'static str = "ClientStateLoggingOut";
        type Type = super::ClientStateLoggingOut;
    }

    impl ObjectImpl for ClientStateLoggingOut {}
}

glib::wrapper! {
    pub(crate) struct ClientStateLoggingOut(ObjectSubclass<imp::ClientStateLoggingOut>);
}

impl Default for ClientStateLoggingOut {
    fn default() -> Self {
        glib::Object::new()
    }
}
