use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdgrand::enums::MessageContent;

use crate::session::chat::{BoxedMessageContent, Message};
use crate::session::components::Avatar;
use crate::session::Chat;
use crate::utils::escape;

fn stringify_message_content(content: MessageContent) -> String {
    match content {
        MessageContent::MessageText(content) => escape(&content.text.text),
        _ => format!("<i>{}</i>", gettext("This message is unsupported")),
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-chat-row.ui")]
    pub struct ChatRow {
        pub chat: RefCell<Option<Chat>>,
        #[template_child]
        pub timestamp_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub last_message_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatRow {
        const NAME: &'static str = "SidebarChatRow";
        type Type = super::ChatRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Avatar::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "chat",
                    "Chat",
                    "The chat represented by this row",
                    Chat::static_type(),
                    glib::ParamFlags::READWRITE,
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
                "chat" => {
                    let chat = value.get().unwrap();
                    obj.set_chat(chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ChatRow {}
    impl BinImpl for ChatRow {}
}

glib::wrapper! {
    pub struct ChatRow(ObjectSubclass<imp::ChatRow>)
        @extends gtk::Widget, adw::Bin;
}

impl ChatRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChatRow")
    }

    pub fn chat(&self) -> Option<Chat> {
        let self_ = imp::ChatRow::from_instance(self);
        self_.chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let self_ = imp::ChatRow::from_instance(self);

        if let Some(ref chat) = chat {
            let chat_expression = gtk::ConstantExpression::new(&chat);
            let last_message_expression = gtk::PropertyExpression::new(
                Chat::static_type(),
                Some(&chat_expression),
                "last-message",
            );

            // Last message timestamp
            let date_expression = gtk::PropertyExpression::new(
                Message::static_type(),
                Some(&last_message_expression),
                "date",
            );
            let timestamp_expression = gtk::ClosureExpression::new(
                move |expressions| -> String {
                    let date = expressions[1].get::<i32>().unwrap();

                    let datetime_now = glib::DateTime::new_now_local().unwrap();
                    let datetime = glib::DateTime::from_unix_utc(date as i64)
                        .and_then(|t| t.to_local())
                        .unwrap();

                    let hours_difference = datetime_now.difference(&datetime) / 3600000000;
                    let days_difference = hours_difference / 24;

                    if hours_difference <= 16 {
                        // Show the time
                        let mut time = datetime.format("%X").unwrap().to_string();

                        // Remove seconds
                        time.replace_range(5..8, "");
                        time
                    } else if days_difference < 6 {
                        // Show the day of the week
                        datetime.format("%a").unwrap().to_string()
                    } else if days_difference < 364 {
                        // Show the day and the month
                        datetime.format("%d %b").unwrap().to_string()
                    } else {
                        // Show the entire date
                        datetime.format("%x").unwrap().to_string()
                    }
                },
                &[date_expression.upcast()],
            );
            let timestamp_label = self_.timestamp_label.get();
            timestamp_expression.bind(&timestamp_label, "label", Some(&timestamp_label));

            // Last message content
            let content_expression = gtk::PropertyExpression::new(
                Message::static_type(),
                Some(&last_message_expression),
                "content",
            );
            let stringified_content_expression = gtk::ClosureExpression::new(
                move |expressions| -> String {
                    let content = expressions[1].get::<BoxedMessageContent>().unwrap();
                    stringify_message_content(content.0)
                },
                &[content_expression.upcast()],
            );
            let last_message_label = self_.last_message_label.get();
            stringified_content_expression.bind(
                &last_message_label,
                "label",
                Some(&last_message_label),
            );
        }

        self_.chat.replace(chat);
        self.notify("chat");
    }
}
