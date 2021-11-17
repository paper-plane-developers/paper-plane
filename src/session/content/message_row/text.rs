use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::enums::{ChatType, MessageContent};

use crate::session::chat::{BoxedMessageContent, Message, MessageSender};
use crate::utils::parse_formatted_text;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-text.ui")]
    pub struct MessageText {
        pub sender_color_class: RefCell<Option<String>>,
        #[template_child]
        pub sender_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub content_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageText {
        const NAME: &'static str = "ContentMessageText";
        type Type = super::MessageText;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageText {
        fn dispose(&self, _obj: &Self::Type) {
            self.sender_label.unparent();
            self.content_label.unparent();
        }
    }

    impl WidgetImpl for MessageText {}
}

glib::wrapper! {
    pub struct MessageText(ObjectSubclass<imp::MessageText>)
        @extends gtk::Widget;
}

impl Default for MessageText {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageText {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageText")
    }

    pub fn set_message(&self, message: &Message) {
        let self_ = imp::MessageText::from_instance(self);

        if message.is_outgoing() {
            self.add_css_class("outgoing");
        } else {
            self.remove_css_class("outgoing");
        }

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
            let sender_name_expression = message.sender_name_expression();
            sender_name_expression.bind(&*self_.sender_label, "label", gtk::NONE_WIDGET);

            // Remove the previous color css class
            let mut sender_color_class = self_.sender_color_class.borrow_mut();
            if let Some(class) = sender_color_class.as_ref() {
                self_.sender_label.remove_css_class(class);
                *sender_color_class = None;
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
                self_.sender_label.add_css_class(color_class);

                *sender_color_class = Some(color_class.into());
            }

            self_.sender_label.set_visible(true);
        } else {
            self_.sender_label.set_visible(false);
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

fn format_message_content_text(content: MessageContent) -> String {
    match content {
        MessageContent::MessageText(content) => parse_formatted_text(content.text),
        _ => format!("<i>{}</i>", gettext("This message is unsupported")),
    }
}
