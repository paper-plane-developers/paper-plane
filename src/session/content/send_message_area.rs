use glib::{clone, signal::Inhibit};
use gtk::{gdk, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums::InputMessageContent, functions, types};

use crate::session::Chat;
use crate::RUNTIME;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-send-message-area.ui")]
    pub struct SendMessageArea {
        pub chat: RefCell<Option<Chat>>,
        #[template_child]
        pub message_entry: TemplateChild<gtk::TextView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SendMessageArea {
        const NAME: &'static str = "ContentSendMessageArea";
        type Type = super::SendMessageArea;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "send-message-area.send-text-message",
                None,
                move |widget, _, _| {
                    widget.send_text_message();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SendMessageArea {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "chat",
                    "Chat",
                    "The chat associated with this widget",
                    Chat::static_type(),
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

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Enable the send-text-message action only when the message entry contains text
            let message_buffer = self.message_entry.buffer();
            message_buffer.connect_text_notify(clone!(@weak obj => move |_| {
                let should_enable = !obj.message_entry_text().is_empty();
                obj.action_set_enabled("send-message-area.send-text-message", should_enable);
            }));

            // The message entry is always empty at this point, so disable the
            // send-text-message action
            obj.action_set_enabled("send-message-area.send-text-message", false);

            // Handle the enter key to send the message and also the combination of if with the
            // right modifier keys to add new lines to the entry
            let key_events = gtk::EventControllerKey::new();
            self.message_entry.add_controller(&key_events);
            key_events.connect_key_pressed(
                clone!(@weak obj => @default-return Inhibit(false), move |_, key, _, modifier| {
                    if !modifier.contains(gdk::ModifierType::SHIFT_MASK)
                        && (key == gdk::keys::constants::Return
                            || key == gdk::keys::constants::KP_Enter)
                    {
                        obj.activate_action("send-message-area.send-text-message", None);
                        Inhibit(true)
                    } else {
                        Inhibit(false)
                    }
                }),
            );
        }
    }

    impl WidgetImpl for SendMessageArea {}
    impl BinImpl for SendMessageArea {}
}

glib::wrapper! {
    pub struct SendMessageArea(ObjectSubclass<imp::SendMessageArea>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for SendMessageArea {
    fn default() -> Self {
        Self::new()
    }
}

impl SendMessageArea {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SendMessageArea")
    }

    fn message_entry_text(&self) -> String {
        let self_ = imp::SendMessageArea::from_instance(self);
        let buffer = self_.message_entry.buffer();
        buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .trim()
            .to_string()
    }

    fn compose_text_message(&self) -> InputMessageContent {
        let text = types::FormattedText {
            text: self.message_entry_text(),
            ..Default::default()
        };
        let content = types::InputMessageText {
            text,
            clear_draft: true,
            ..Default::default()
        };

        InputMessageContent::InputMessageText(content)
    }

    fn send_text_message(&self) {
        if let Some(chat) = self.chat() {
            let message = self.compose_text_message();
            let client_id = chat.session().client_id();
            let chat_id = chat.id();

            // Send the message
            RUNTIME.spawn(async move {
                functions::SendMessage::new()
                    .chat_id(chat_id)
                    .input_message_content(message)
                    .send(client_id)
                    .await
                    .unwrap();
            });

            // Reset message entry
            let self_ = imp::SendMessageArea::from_instance(self);
            let buffer = self_.message_entry.buffer();
            buffer.set_text("");
        }
    }

    fn save_message_as_draft(&self) {
        if let Some(chat) = self.chat() {
            let message = self.compose_text_message();
            let draft_message = types::DraftMessage {
                input_message_text: message,
                ..Default::default()
            };
            let client_id = chat.session().client_id();
            let chat_id = chat.id();

            // Save draft message
            RUNTIME.spawn(async move {
                functions::SetChatDraftMessage::new()
                    .chat_id(chat_id)
                    .draft_message(draft_message)
                    .send(client_id)
                    .await
                    .unwrap();
            });
        }
    }

    fn load_draft_message(&self, message: String) {
        let self_ = imp::SendMessageArea::from_instance(self);
        self_.message_entry.buffer().set_text(&message);
    }

    pub fn chat(&self) -> Option<Chat> {
        let self_ = imp::SendMessageArea::from_instance(self);
        self_.chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        self.save_message_as_draft();

        if let Some(ref chat) = chat {
            self.load_draft_message(chat.draft_message());
        }

        let self_ = imp::SendMessageArea::from_instance(self);
        self_.chat.replace(chat);
        self.notify("chat");
    }
}
