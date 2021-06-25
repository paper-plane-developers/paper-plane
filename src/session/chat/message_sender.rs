use crate::session::{Chat, User};

#[derive(Debug, Clone)]
pub enum MessageSender {
    Chat(Chat),
    User(User),
}
