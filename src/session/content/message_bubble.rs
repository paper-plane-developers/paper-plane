use adw::{prelude::BinExt, subclass::prelude::BinImpl};
use gettextrs::gettext;
use gtk::{glib, pango, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::enums::{ChatType, MessageContent, TextEntityType};
use tdgrand::types::FormattedText;

use crate::session::chat::{BoxedMessageContent, Message, MessageSender};
use crate::utils::{escape, linkify};

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-bubble.ui")]
    pub struct MessageBubble {
        pub is_outgoing: Cell<bool>,
        pub sender_color_class: RefCell<Option<String>>,
        #[template_child]
        pub sender_bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub content_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageBubble {
        const NAME: &'static str = "ContentMessageBubble";
        type Type = super::MessageBubble;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageBubble {}
    impl WidgetImpl for MessageBubble {}
    impl BinImpl for MessageBubble {}
}

glib::wrapper! {
    pub struct MessageBubble(ObjectSubclass<imp::MessageBubble>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for MessageBubble {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBubble {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageBubble")
    }

    pub fn set_message(&self, message: &Message) {
        let self_ = imp::MessageBubble::from_instance(self);

        // Remove previous css class
        if self_.is_outgoing.get() {
            self.remove_css_class("outgoing")
        } else {
            self.remove_css_class("incoming")
        }

        // Set new css class
        if message.is_outgoing() {
            self.add_css_class("outgoing");
        } else {
            self.add_css_class("incoming");
        }
        self_.is_outgoing.set(message.is_outgoing());

        // Show sender label, if needed
        let show_sender = {
            if !message.is_outgoing() {
                matches!(
                    message.chat().type_(),
                    ChatType::BasicGroup(_) | ChatType::Supergroup(_)
                )
            } else {
                false
            }
        };
        if show_sender {
            let label = if let Some(Ok(label)) =
                self_.sender_bin.child().map(|w| w.downcast::<gtk::Label>())
            {
                label
            } else {
                let label = gtk::LabelBuilder::new()
                    .css_classes(vec!["sender-text".to_string()])
                    .halign(gtk::Align::Start)
                    .ellipsize(pango::EllipsizeMode::End)
                    .single_line_mode(true)
                    .build();
                self_.sender_bin.set_child(Some(&label));
                label
            };
            let sender_name_expression = message.sender_name_expression();
            sender_name_expression.bind(&label, "label", Some(&label));

            // Remove the previous color css class
            if let Some(class) = self_.sender_color_class.borrow().as_ref() {
                label.remove_css_class(class);
            }

            // Color sender label
            if let MessageSender::User(user) = message.sender() {
                let classes = vec![
                    "sender-text-red",
                    "sender-text-orange",
                    "sender-text-violet",
                    "sender-text-green",
                    "sender-text-cyan",
                    "sender-text-blue",
                    "sender-text-pink",
                ];

                let color_class = classes[user.id() as usize % classes.len()];
                label.add_css_class(color_class);

                self_.sender_color_class.replace(Some(color_class.into()));
            } else {
                self_.sender_color_class.replace(None);
            }
        } else {
            self_.sender_bin.set_child(None::<&gtk::Widget>);
            self_.sender_color_class.replace(None);
        }

        // Set content label expression
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
        let content_label = self_.content_label.get();
        text_expression.bind(&content_label, "label", Some(&content_label));
    }
}

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
    let mut is_inside_entity = false;

    // This is the offset in utf16 code units of the text to parse. We need this variable
    // because tdlib stores the offset and length parameters as utf16 code units instead
    // of regular code points.
    let mut code_units_offset = 0;

    for c in formatted_text.text.chars() {
        if !is_inside_entity
            && entity.is_some()
            && code_units_offset >= entity.unwrap().offset as usize
        {
            is_inside_entity = true;

            if !buffer.is_empty() {
                output.push_str(&escape(&buffer));
                buffer = String::new();
            }
        }

        buffer.push(c);
        code_units_offset += c.len_utf16();

        if let Some(entity_) = entity {
            if code_units_offset >= (entity_.offset + entity_.length) as usize {
                buffer = escape(&buffer);

                entity = loop {
                    let entity = entities.next();

                    // Handle eventual nested entities
                    match entity {
                        Some(entity) => {
                            if entity.offset == entity_.offset {
                                buffer = convert_to_markup(buffer, &entity.r#type);
                            } else {
                                break Some(entity);
                            }
                        }
                        None => break None,
                    }
                };

                output.push_str(&convert_to_markup(buffer, &entity_.r#type));
                buffer = String::new();
                is_inside_entity = false;
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
