use glib::{clone, signal::Inhibit};
use gtk::{gdk, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{
    enums::{ChatAction, InputMessageContent},
    functions, types,
};

use crate::session::Chat;
use crate::utils::do_async;
use crate::RUNTIME;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-action-bar.ui")]
    pub struct ChatActionBar {
        pub chat: RefCell<Option<Chat>>,
        pub chat_action_in_cooldown: Cell<bool>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub message_entry: TemplateChild<gtk::TextView>,
        #[template_child]
        pub send_message_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatActionBar {
        const NAME: &'static str = "ContentChatActionBar";
        type Type = super::ChatActionBar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "chat-action-bar.send-text-message",
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

    impl ObjectImpl for ChatActionBar {
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

            let message_buffer = self.message_entry.buffer();
            message_buffer.connect_text_notify(clone!(@weak obj => move |_| {
                // Enable the send-text-message action only when the message entry contains text
                let should_enable = !obj.message_entry_text().is_empty();
                obj.action_set_enabled("chat-action-bar.send-text-message", should_enable);

                // Send typing action
                obj.send_chat_action(ChatAction::Typing);
            }));

            // The message entry is always empty at this point, so disable the
            // send-text-message action
            obj.action_set_enabled("chat-action-bar.send-text-message", false);

            // Handle the enter key to send the message and also the combination of if with the
            // right modifier keys to add new lines to the entry
            let key_events = gtk::EventControllerKey::new();
            self.message_entry.add_controller(&key_events);
            key_events.connect_key_pressed(
                clone!(@weak obj => @default-return Inhibit(false), move |_, key, _, modifier| {
                    if !modifier.contains(gdk::ModifierType::CONTROL_MASK)
                        && !modifier.contains(gdk::ModifierType::SHIFT_MASK)
                        && (key == gdk::keys::constants::Return
                            || key == gdk::keys::constants::KP_Enter)
                    {
                        obj.activate_action("chat-action-bar.send-text-message", None);
                        Inhibit(true)
                    } else {
                        Inhibit(false)
                    }
                }),
            );
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.scrolled_window.unparent();
            self.send_message_button.unparent();
        }
    }

    impl WidgetImpl for ChatActionBar {}
}

glib::wrapper! {
    pub struct ChatActionBar(ObjectSubclass<imp::ChatActionBar>)
        @extends gtk::Widget;
}

impl Default for ChatActionBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatActionBar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChatActionBar")
    }

    fn message_entry_text(&self) -> String {
        let self_ = imp::ChatActionBar::from_instance(self);
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
            let self_ = imp::ChatActionBar::from_instance(self);
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
        let self_ = imp::ChatActionBar::from_instance(self);
        self_.message_entry.buffer().set_text(&message);
    }

    fn send_chat_action(&self, action: ChatAction) {
        let self_ = imp::ChatActionBar::from_instance(self);
        if self_.chat_action_in_cooldown.get() {
            return;
        }

        if let Some(chat) = self.chat() {
            let client_id = chat.session().client_id();
            let chat_id = chat.id();

            // Enable chat action cooldown right away
            self_.chat_action_in_cooldown.set(true);

            // Send typing action
            do_async(
                glib::PRIORITY_DEFAULT_IDLE,
                async move {
                    functions::SendChatAction::new()
                        .chat_id(chat_id)
                        .action(action)
                        .send(client_id)
                        .await
                },
                clone!(@weak self as obj => move |result| async move {
                    // If the request is successful, then start the actual cooldown of 5 seconds.
                    // Otherwise just cancel it right away.
                    if result.is_ok() {
                        glib::timeout_add_seconds_local_once(5, clone!(@weak obj =>move || {
                            let self_ = imp::ChatActionBar::from_instance(&obj);
                            self_.chat_action_in_cooldown.set(false);
                        }));
                    } else {
                        let self_ = imp::ChatActionBar::from_instance(&obj);
                        self_.chat_action_in_cooldown.set(false);
                    }
                }),
            );
        }
    }

    pub fn chat(&self) -> Option<Chat> {
        let self_ = imp::ChatActionBar::from_instance(self);
        self_.chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        self.save_message_as_draft();

        let self_ = imp::ChatActionBar::from_instance(self);

        if let Some(ref chat) = chat {
            self.load_draft_message(chat.draft_message());

            self_.chat_action_in_cooldown.set(false);
        }

        self_.chat.replace(chat);
        self.notify("chat");
    }
}
