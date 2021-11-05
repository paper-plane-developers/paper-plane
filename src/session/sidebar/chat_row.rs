use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*};
use tdgrand::enums::{ChatType, MessageContent};

use crate::session::chat::{BoxedChatNotificationSettings, Message, MessageSender};
use crate::session::components::Avatar;
use crate::session::Chat;
use crate::utils::{dim_and_escape, escape};

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
        #[template_child]
        pub pin_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub unread_count_label: TemplateChild<gtk::Label>,
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

impl Default for ChatRow {
    fn default() -> Self {
        Self::new()
    }
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

            // Last message label
            //
            // FIXME: The sender name should be part of the expressions, but it can't because
            // the sender object is an enum of two object variants and there's no way to obtain
            // one of the two objects from a message expression to make a sender variant expression.
            let content_expression = gtk::PropertyExpression::new(
                Message::static_type(),
                Some(&last_message_expression),
                "content",
            );
            let stringified_message_expression = gtk::ClosureExpression::new(
                move |args| {
                    let last_message = args[1].get::<Message>().unwrap();
                    stringify_message(last_message)
                },
                &[
                    last_message_expression.upcast(),
                    content_expression.upcast(),
                ],
            );
            let last_message_label = self_.last_message_label.get();
            stringified_message_expression.bind(
                &last_message_label,
                "label",
                Some(&last_message_label),
            );

            // Pinned icon and unread badge visibility
            let is_pinned_expression = gtk::PropertyExpression::new(
                Chat::static_type(),
                Some(&chat_expression),
                "is-pinned",
            );
            let unread_count_expression = gtk::PropertyExpression::new(
                Chat::static_type(),
                Some(&chat_expression),
                "unread-count",
            );
            let pin_visibility_expression = gtk::ClosureExpression::new(
                move |args| {
                    let is_pinned = args[1].get::<bool>().unwrap();
                    let unread_count = args[2].get::<i32>().unwrap();

                    is_pinned && unread_count <= 0
                },
                &[
                    is_pinned_expression.upcast(),
                    unread_count_expression.upcast(),
                ],
            );
            let pin_image = self_.pin_image.get();
            pin_visibility_expression.bind(&pin_image, "visible", Some(&pin_image));

            let notification_settings_expression = gtk::PropertyExpression::new(
                Chat::static_type(),
                Some(&chat_expression),
                "notification-settings",
            );
            let unread_count_label_css_expression = gtk::ClosureExpression::new(
                |args| {
                    let notification_settings =
                        args[1].get::<BoxedChatNotificationSettings>().unwrap().0;

                    vec![
                        "unread-count".to_string(),
                        if notification_settings.mute_for > 0 {
                            "unread-count-muted"
                        } else {
                            "unread-count-unmuted"
                        }
                        .to_string(),
                    ]
                },
                &[notification_settings_expression.upcast()],
            );
            unread_count_label_css_expression.bind(
                &*self_.unread_count_label,
                "css-classes",
                gtk::NONE_WIDGET,
            );
        }

        self_.chat.replace(chat);
        self.notify("chat");
    }
}

fn stringify_message(message: Message) -> String {
    let mut show_sender = match message.chat().type_() {
        ChatType::BasicGroup(_) => true,
        ChatType::Supergroup(data) => !data.is_channel,
        ChatType::Private(_) | ChatType::Secret(_) => message.is_outgoing(),
    };

    let text_content = match message.content().0 {
        MessageContent::MessageText(data) => dim_and_escape(&data.text.text),
        MessageContent::MessageBasicGroupChatCreate(_) => {
            show_sender = false;
            gettext!("{} created the group", sender_name(message.sender(), true))
        }
        MessageContent::MessageSticker(data) => {
            format!("{} {}", data.sticker.emoji, gettext("Sticker"))
        }
        MessageContent::MessagePhoto(data) => format!(
            "{}{}",
            gettext("Photo"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", dim_and_escape(&data.caption.text))
            }
        ),
        MessageContent::MessageAudio(data) => format!(
            "{} - {}{}",
            data.audio.performer,
            data.audio.title,
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", dim_and_escape(&data.caption.text))
            }
        ),
        MessageContent::MessageAnimation(data) => format!(
            "{}{}",
            gettext("GIF"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", dim_and_escape(&data.caption.text))
            }
        ),
        MessageContent::MessageVideo(data) => format!(
            "{}{}",
            gettext("Video"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", dim_and_escape(&data.caption.text))
            }
        ),
        MessageContent::MessageDocument(data) => format!(
            "{}{}",
            data.document.file_name,
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", dim_and_escape(&data.caption.text))
            }
        ),
        MessageContent::MessageVoiceNote(data) => format!(
            "{}{}",
            gettext("Voice message"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", dim_and_escape(&data.caption.text))
            }
        ),
        MessageContent::MessageChatDeletePhoto => {
            show_sender = false;

            match message.chat().type_() {
                ChatType::Supergroup(data) if data.is_channel => gettext("Channel photo removed"),
                _ => {
                    if message.is_outgoing() {
                        gettext("You removed the group photo")
                    } else {
                        gettext!(
                            "{} removed the group photo",
                            sender_name(message.sender(), true)
                        )
                    }
                }
            }
        }
        MessageContent::MessageContactRegistered => {
            gettext!("{} joined Telegram", sender_name(message.sender(), true))
        }
        _ => gettext("Unsupported message"),
    };

    if show_sender {
        let sender_name = if message.is_outgoing() {
            gettext("You")
        } else {
            escape(&sender_name(message.sender(), false))
        };

        format!("{}: {}", sender_name, text_content)
    } else {
        text_content
    }
}

fn sender_name(sender: &MessageSender, use_full_name: bool) -> String {
    match sender {
        MessageSender::User(user) => {
            if use_full_name {
                format!("{} {}", user.first_name(), user.last_name())
                    .trim()
                    .into()
            } else {
                user.first_name()
            }
        }
        MessageSender::Chat(chat) => chat.title(),
    }
}
