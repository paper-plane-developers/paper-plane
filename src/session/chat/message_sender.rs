use gtk::glib;

use crate::session::{Chat, User};

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "MessageSender")]
pub enum MessageSender {
    Chat(Chat),
    User(User),
}
