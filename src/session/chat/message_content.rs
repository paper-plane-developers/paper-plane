use tdgrand::enums::MessageContent as TelegramMessageContent;

#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(String),
    Unsupported,
}

impl MessageContent {
    pub fn new(content: TelegramMessageContent) -> Self {
        return match content {
            TelegramMessageContent::MessageText(content) =>
                MessageContent::Text(content.text.text),
            _ => MessageContent::Unsupported,
        }
    }
}
