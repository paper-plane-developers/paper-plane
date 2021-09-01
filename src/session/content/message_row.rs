use adw::prelude::BinExt;
use gettextrs::gettext;
use gtk::{glib, pango, prelude::*, subclass::prelude::*};
use tdgrand::enums::{ChatType, MessageContent, TextEntityType};
use tdgrand::types::FormattedText;

use crate::session::chat::{BoxedMessageContent, Message, MessageSender};
use crate::session::components::Avatar;
use crate::session::{Chat, User};
use crate::utils::{escape, linkify};

fn convert_to_markup(text: String, entity: &TextEntityType) -> String {
    match entity {
        TextEntityType::Url => format!("<a href='{}'>{}</a>", linkify(&text), text),
        TextEntityType::EmailAddress => format!("<a href='mailto:{0}'>{0}</a>", text),
        TextEntityType::PhoneNumber => format!("<a href='tel:{0}'>{0}</a>", text),
        TextEntityType::Bold => format!("<b>{}</b>", text),
        TextEntityType::Italic => format!("<i>{}</i>", text),
        TextEntityType::Underline => format!("<u>{}</u>", text),
        TextEntityType::Strikethrough => format!("<s>{}</s>", text),
        TextEntityType::Code | TextEntityType::Pre | TextEntityType::PreCode(_) => {
            format!("<tt>{}</tt>", text)
        }
        TextEntityType::TextUrl(data) => format!("<a href='{}'>{}</a>", escape(&data.url), text),
        _ => text,
    }
}

fn parse_formatted_text(formatted_text: FormattedText) -> String {
    let mut entities = formatted_text.entities.iter();
    let mut entity = entities.next();
    let mut output = String::new();
    let mut buffer = String::new();
    let mut inside_entry = false;

    // This is the offset in utf16 code units of the text to parse. We need this variable
    // because tdlib stores the offset and length parameters as utf16 code units instead
    // of regular code points.
    let mut code_units_offset = 0;

    for c in formatted_text.text.chars() {
        if !inside_entry && entity.is_some() && code_units_offset >= entity.unwrap().offset as usize
        {
            inside_entry = true;

            if !buffer.is_empty() {
                output.push_str(&escape(&buffer));
                buffer = String::new();
            }
        }

        buffer.push(c);
        code_units_offset = code_units_offset + c.len_utf16();

        if let Some(entity_) = entity {
            if code_units_offset >= (entity_.offset + entity_.length) as usize {
                buffer = escape(&buffer);

                entity = loop {
                    let entity = entities.next();

                    // Handle eventual nested entries
                    if entity.is_some() && entity.unwrap().offset == entity_.offset {
                        buffer = convert_to_markup(buffer, &entity.unwrap().r#type);
                    } else {
                        break entity;
                    }
                };

                output.push_str(&convert_to_markup(buffer, &entity_.r#type));
                buffer = String::new();
                inside_entry = false;
            }
        }
    }

    // Add the eventual leftovers from the buffer to the output
    if !buffer.is_empty() {
        output.push_str(&escape(&buffer));
    }

    output
}

fn format_message_content_text(content: MessageContent) -> String {
    match content {
        MessageContent::MessageText(content) => parse_formatted_text(content.text),
        _ => format!("<i>{}</i>", gettext("This message is unsupported")),
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct MessageRow {
        pub message: RefCell<Option<Message>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "ContentMessageRow";
        type Type = super::MessageRow;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for MessageRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "message",
                    "Message",
                    "The message represented by this row",
                    Message::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "message" => {
                    let message = value.get().unwrap();
                    obj.set_message(message);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => obj.message().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageRow {}
    impl BinImpl for MessageRow {}
}

glib::wrapper! {
    pub struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk::Widget, adw::Bin;
}

impl MessageRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageRow")
    }

    pub fn message(&self) -> Option<Message> {
        let self_ = imp::MessageRow::from_instance(self);
        self_.message.borrow().clone()
    }

    pub fn set_message(&self, message: Message) {
        self.show_message_bubble(&message);

        let self_ = imp::MessageRow::from_instance(self);
        self_.message.replace(Some(message));
        self.notify("message");
    }

    fn show_message_bubble(&self, message: &Message) {
        let hbox = gtk::BoxBuilder::new().spacing(6).build();

        let vbox = gtk::BoxBuilder::new()
            .css_classes(vec!["message-bubble".to_string()])
            .orientation(gtk::Orientation::Vertical)
            .build();
        hbox.append(&vbox);

        if message.outgoing() {
            hbox.set_halign(gtk::Align::End);
            vbox.add_css_class("outgoing");
        } else {
            hbox.set_halign(gtk::Align::Start);
            vbox.add_css_class("incoming");
        }

        if !message.outgoing() {
            let is_channel = if let ChatType::Supergroup(data) = message.chat().r#type() {
                data.is_channel
            } else {
                false
            };

            match message.chat().r#type() {
                ChatType::BasicGroup(_) | ChatType::Supergroup(_) => {
                    let sender_label = MessageRow::create_sender_label(message);
                    vbox.append(&sender_label);

                    if !is_channel {
                        let sender_avatar = MessageRow::create_sender_avatar(message);
                        hbox.prepend(&sender_avatar);

                        sender_label
                            .bind_property("label", &sender_avatar, "display-name")
                            .flags(glib::BindingFlags::SYNC_CREATE)
                            .build();
                    }
                }
                _ => {}
            }
        }

        let text_label = MessageRow::create_text_label(message);
        vbox.append(&text_label);

        self.set_child(Some(&hbox));
    }

    fn create_sender_label(message: &Message) -> gtk::Label {
        let label = gtk::LabelBuilder::new()
            .css_classes(vec!["sender-text".to_string()])
            .ellipsize(pango::EllipsizeMode::End)
            .single_line_mode(true)
            .xalign(0.0)
            .build();

        match message.sender() {
            MessageSender::User(user) => {
                let user_expression = gtk::ConstantExpression::new(&user);
                let first_name_expression = gtk::PropertyExpression::new(
                    User::static_type(),
                    Some(&user_expression),
                    "first-name",
                );
                let last_name_expression = gtk::PropertyExpression::new(
                    User::static_type(),
                    Some(&user_expression),
                    "last-name",
                );
                let full_name_expression = gtk::ClosureExpression::new(
                    move |expressions| -> String {
                        let first_name = expressions[1].get::<&str>().unwrap();
                        let last_name = expressions[2].get::<&str>().unwrap();
                        format!("{} {}", first_name, last_name).trim().to_string()
                    },
                    &[
                        first_name_expression.upcast(),
                        last_name_expression.upcast(),
                    ],
                );

                full_name_expression.bind(&label, "label", Some(&label));
            }
            MessageSender::Chat(chat) => {
                let chat_expression = gtk::ConstantExpression::new(&chat);
                let title_expression = gtk::PropertyExpression::new(
                    Chat::static_type(),
                    Some(&chat_expression),
                    "title",
                );

                title_expression.bind(&label, "label", Some(&label));
            }
        }

        label
    }

    fn create_sender_avatar(message: &Message) -> Avatar {
        let sender_avatar = Avatar::new();
        sender_avatar.set_size(32);
        sender_avatar.set_valign(gtk::Align::End);

        match message.sender() {
            MessageSender::User(user) => sender_avatar.set_item(Some(user.avatar().clone())),
            MessageSender::Chat(chat) => sender_avatar.set_item(Some(chat.avatar().clone())),
        }

        sender_avatar
    }

    fn create_text_label(message: &Message) -> gtk::Label {
        let label = gtk::LabelBuilder::new()
            .css_classes(vec!["message-text".to_string()])
            .selectable(true)
            .use_markup(true)
            .wrap(true)
            .wrap_mode(pango::WrapMode::WordChar)
            .xalign(0.0)
            .build();

        let message_expression = gtk::ConstantExpression::new(message);
        let content_expression = gtk::PropertyExpression::new(
            Message::static_type(),
            Some(&message_expression),
            "content",
        );
        let text_expression = gtk::ClosureExpression::new(
            move |expressions| -> String {
                let content = expressions[1].get::<BoxedMessageContent>().unwrap();
                format_message_content_text(content.0)
            },
            &[content_expression.upcast()],
        );
        text_expression.bind(&label, "label", Some(&label));

        label
    }
}
