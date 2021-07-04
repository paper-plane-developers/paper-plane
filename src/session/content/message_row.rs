use adw::prelude::BinExt;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};

use crate::session::{Chat, User};
use crate::session::chat::{Message, MessageContent, MessageSender};

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
                vec![
                    glib::ParamSpec::new_object(
                        "message",
                        "Message",
                        "The message represented by this row",
                        Message::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
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
        let priv_ = imp::MessageRow::from_instance(self);
        priv_.message.borrow().clone()
    }

    fn set_message(&self, message: Option<Message>) {
        if self.message() == message {
            return;
        }

        let priv_ = imp::MessageRow::from_instance(self);

        if let Some(ref message) = message {
            let message_bubble = MessageRow::create_message_bubble(message);
            self.set_child(Some(&message_bubble));
        }

        priv_.message.replace(message);
        self.notify("message");
    }

    fn create_message_bubble(message: &Message) -> gtk::Box {
        let hbox = gtk::BoxBuilder::new()
            .spacing(6)
            .build();

        let vbox = gtk::BoxBuilder::new()
            .css_classes(vec!("message-bubble".to_string()))
            .orientation(gtk::Orientation::Vertical)
            .build();
        hbox.append(&vbox);

        if message.outgoing() {
            hbox.set_halign(gtk::Align::End);
            vbox.add_css_class("outgoing");
        } else {
            hbox.set_halign(gtk::Align::Start);
            vbox.add_css_class("incoming");

            let sender_label = gtk::LabelBuilder::new()
                .css_classes(vec!("sender".to_string()))
                .single_line_mode(true)
                .xalign(0.0)
                .build();
            vbox.append(&sender_label);

            match message.sender() {
                MessageSender::Chat(chat) => {
                    let chat_expression = gtk::ConstantExpression::new(&chat);
                    let title_expression = gtk::PropertyExpression::new(
                        Chat::static_type(),
                        Some(&chat_expression),
                        "title",
                    );
                    title_expression.bind(&sender_label, "label", Some(&sender_label));
                },
                MessageSender::User(user) => {
                    let avatar = adw::AvatarBuilder::new()
                        .valign(gtk::Align::End)
                        .show_initials(true)
                        .size(32)
                        .build();
                    hbox.prepend(&avatar);

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
                        &[first_name_expression.upcast(), last_name_expression.upcast()]
                    );

                    full_name_expression.bind(&avatar, "text", Some(&avatar));
                    full_name_expression.bind(&sender_label, "label", Some(&sender_label));
                },
            }
        }

        let text = match message.content() {
            MessageContent::Text(text) => {
                text.to_owned()
            },
            MessageContent::Unsupported => {
                format!("<i>{}</i>", gettext("This message is unsupported"))
            },
        };

        let text_label = gtk::LabelBuilder::new()
            .css_classes(vec!("message-content".to_string()))
            .vexpand(true)
            .label(&text)
            .selectable(true)
            .use_markup(true)
            .wrap(true)
            .wrap_mode(pango::WrapMode::WordChar)
            .xalign(0.0)
            .build();
        vbox.append(&text_label);

        hbox
    }
}
