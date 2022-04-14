use anyhow::anyhow;
use ashpd::desktop::file_chooser::{FileChooserProxy, FileFilter, OpenFileOptions};
use ashpd::{zbus, WindowIdentifier};
use gettextrs::gettext;
use glib::clone;
use glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};
use tdlib::enums::{ChatAction, InputMessageContent};
use tdlib::{functions, types};

use crate::session::chat::BoxedDraftMessage;
use crate::session::components::{BoxedFormattedText, MessageEntry};
use crate::session::content::SendPhotoDialog;
use crate::session::Chat;
use crate::utils::{spawn, temp_dir};

const PHOTO_MIME_TYPES: &[&str] = &["image/png", "image/jpeg"];

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-action-bar.ui")]
    pub(crate) struct ChatActionBar {
        pub(super) chat: RefCell<Option<Chat>>,
        pub(super) chat_action_in_cooldown: Cell<bool>,
        pub(super) emoji_chooser: RefCell<Option<gtk::EmojiChooser>>,
        #[template_child]
        pub(super) message_entry: TemplateChild<MessageEntry>,
        #[template_child]
        pub(super) send_message_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatActionBar {
        const NAME: &'static str = "ContentChatActionBar";
        type Type = super::ChatActionBar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("chat-action-bar.select-file", None, move |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    widget.select_file().await;
                }));
            });
            klass.install_action(
                "chat-action-bar.send-text-message",
                None,
                move |widget, _, _| {
                    spawn(clone!(@weak widget => async move {
                        widget.send_text_message().await;
                    }));
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
                vec![glib::ParamSpecObject::new(
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

            self.message_entry.connect_formatted_text_notify(
                clone!(@weak obj => move |message_entry, _| {
                    // Enable the send-text-message action only when the message entry contains
                    // at least one non-whitespace character
                    let should_enable = message_entry
                        .formatted_text()
                        .map(|f| f.0.text.contains(|c: char| !c.is_whitespace()))
                        .unwrap_or_default();
                    obj.action_set_enabled("chat-action-bar.send-text-message", should_enable);

                    // Send typing action
                    spawn(clone!(@weak obj => async move {
                        obj.send_chat_action(ChatAction::Typing).await;
                    }));
                }),
            );

            self.message_entry
                .connect_paste_clipboard(clone!(@weak obj => move |_| {
                    obj.handle_paste_action();
                }));

            self.message_entry
                .connect_emoji_button_press(clone!(@weak obj => move |_, button| {
                    obj.show_emoji_chooser(&button);
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
                        && (key == gdk::Key::Return
                            || key == gdk::Key::KP_Enter)
                    {
                        obj.activate_action("chat-action-bar.send-text-message", None).unwrap();
                        Inhibit(true)
                    } else {
                        Inhibit(false)
                    }
                }),
            );
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.message_entry.unparent();
            self.send_message_button.unparent();
            if let Some(emoji_chooser) = self.emoji_chooser.take() {
                emoji_chooser.unparent();
            }
        }
    }

    impl WidgetImpl for ChatActionBar {}
}

glib::wrapper! {
    pub(crate) struct ChatActionBar(ObjectSubclass<imp::ChatActionBar>)
        @extends gtk::Widget;
}

impl Default for ChatActionBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatActionBar {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChatActionBar")
    }

    fn compose_text_message(&self) -> Option<InputMessageContent> {
        if let Some(formatted_text) = self.imp().message_entry.formatted_text() {
            let content = types::InputMessageText {
                text: formatted_text.0,
                disable_web_page_preview: false,
                clear_draft: true,
            };

            Some(InputMessageContent::InputMessageText(content))
        } else {
            None
        }
    }

    fn show_emoji_chooser(&self, parent: &impl IsA<gtk::Widget>) {
        let imp = self.imp();
        let mut emoji_chooser = imp.emoji_chooser.borrow_mut();
        if emoji_chooser.is_none() {
            let chooser = gtk::EmojiChooser::new();
            chooser.set_parent(parent);
            chooser.connect_emoji_picked(clone!(@weak self as obj => move |_, emoji| {
                obj.imp().message_entry.insert_at_cursor(emoji);
            }));
            *emoji_chooser = Some(chooser);
        }
        emoji_chooser.as_ref().unwrap().popup();
    }

    async fn select_file(&self) {
        let connection = zbus::Connection::session().await.unwrap();
        let proxy = FileChooserProxy::new(&connection).await.unwrap();
        let native = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&native).await;
        let mut filter = FileFilter::new(&gettext("Image"));

        for mime in PHOTO_MIME_TYPES {
            filter = filter.mimetype(mime);
        }

        let options = OpenFileOptions::default().modal(true).add_filter(filter);

        if let Ok(files) = proxy
            .open_file(&identifier, &gettext("Select File"), options)
            .await
        {
            let parent_window = self.root().unwrap().downcast().ok();
            let chat = self.chat().unwrap();
            let file = gio::File::for_uri(&files.uris()[0]);

            if let Some(path) = file.path() {
                let path = path.to_str().unwrap().to_string();
                SendPhotoDialog::new(&parent_window, chat, path).present();
            }
        }
    }

    async fn send_text_message(&self) {
        if let Some(chat) = self.chat() {
            if let Some(message) = self.compose_text_message() {
                let client_id = chat.session().client_id();
                let chat_id = chat.id();

                // Send the message
                let result = functions::send_message(chat_id, 0, 0, None, message, client_id).await;
                if let Err(e) = result {
                    log::warn!("Error sending a message: {:?}", e);
                }

                // Reset message entry
                self.imp().message_entry.set_formatted_text(None);
            }
        }
    }

    async fn save_message_as_draft(&self) {
        if let Some(chat) = self.chat() {
            let client_id = chat.session().client_id();
            let chat_id = chat.id();
            let draft_message = self
                .compose_text_message()
                .map(|message| types::DraftMessage {
                    reply_to_message_id: 0,
                    date: glib::DateTime::now_local().unwrap().to_unix() as i32,
                    input_message_text: message,
                });

            // Save draft message
            let result =
                functions::set_chat_draft_message(chat_id, 0, draft_message, client_id).await;
            if let Err(e) = result {
                log::warn!("Error setting a draft message: {:?}", e);
            }
        }
    }

    fn load_draft_message(&self, message: Option<BoxedDraftMessage>) {
        let formatted_text = if let Some(message) = message {
            if let InputMessageContent::InputMessageText(content) = message.0.input_message_text {
                Some(BoxedFormattedText(content.text))
            } else {
                log::warn!(
                    "Unexpected draft message type: {:?}",
                    message.0.input_message_text
                );
                None
            }
        } else {
            None
        };

        self.imp().message_entry.set_formatted_text(formatted_text);
    }

    async fn send_chat_action(&self, action: ChatAction) {
        let imp = self.imp();
        if imp.chat_action_in_cooldown.get() {
            return;
        }

        if let Some(chat) = self.chat() {
            let client_id = chat.session().client_id();
            let chat_id = chat.id();

            // Enable chat action cooldown right away
            imp.chat_action_in_cooldown.set(true);

            // Send typing action
            let result = functions::send_chat_action(chat_id, 0, Some(action), client_id).await;
            if result.is_ok() {
                glib::timeout_add_seconds_local_once(
                    5,
                    clone!(@weak self as obj =>move || {
                        obj.imp().chat_action_in_cooldown.set(false);
                    }),
                );
            } else {
                imp.chat_action_in_cooldown.set(false);
            }
        }
    }

    pub(crate) fn handle_paste_action(&self) {
        if let Some(chat) = self.chat() {
            spawn(clone!(@weak self as obj => async move {
                if let Err(e) = obj.handle_image_clipboard(chat).await {
                    log::warn!("Error on pasting an image: {:?}", e);
                }
            }));
        }
    }

    async fn handle_image_clipboard(&self, chat: Chat) -> Result<(), anyhow::Error> {
        if let Ok((stream, mime)) = self
            .clipboard()
            .read_future(PHOTO_MIME_TYPES, glib::PRIORITY_DEFAULT)
            .await
        {
            let extension = match mime.as_str() {
                "image/png" => "png",
                "image/jpg" => "jpg",
                _ => unreachable!(),
            };

            let temp_dir =
                temp_dir().ok_or_else(|| anyhow!("The temporary directory doesn't exist"))?;
            let path = temp_dir.join("clipboard").with_extension(extension);

            save_stream_to_file(stream, &path).await?;

            let parent_window = self.root().unwrap().downcast().ok();
            let path = path.to_str().unwrap().to_string();
            SendPhotoDialog::new(&parent_window, chat, path).present();
        }

        Ok(())
    }

    pub(crate) fn chat(&self) -> Option<Chat> {
        self.imp().chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        spawn(clone!(@weak self as obj => async move {
            obj.save_message_as_draft().await;
        }));

        let imp = self.imp();

        if let Some(ref chat) = chat {
            self.load_draft_message(chat.draft_message());

            imp.chat_action_in_cooldown.set(false);
        }

        imp.chat.replace(chat);
        self.notify("chat");
    }
}

async fn save_stream_to_file(
    stream: gio::InputStream,
    path: impl AsRef<std::path::Path>,
) -> Result<(), glib::Error> {
    let file = gio::File::for_path(path);
    let file_stream = file
        .replace_future(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            glib::PRIORITY_DEFAULT,
        )
        .await?;

    file_stream
        .splice_future(
            &stream,
            gio::OutputStreamSpliceFlags::CLOSE_SOURCE | gio::OutputStreamSpliceFlags::CLOSE_TARGET,
            glib::PRIORITY_DEFAULT,
        )
        .await?;

    Ok(())
}
