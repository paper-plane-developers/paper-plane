use gtk::glib;
use tdgrand::enums::MessageContent as TelegramMessageContent;

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "MessageContent")]
pub enum MessageContent {
    Text(String),
    Unsupported,
}

impl MessageContent {
    pub fn new(content: TelegramMessageContent) -> Self {
        match content {
            TelegramMessageContent::MessageText(content) =>
                MessageContent::Text(content.text.text),
            _ => MessageContent::Unsupported,
        }
    }
}
