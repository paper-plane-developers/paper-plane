use gettextrs::gettext;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use std::thread;
use tdlib::enums::{self, MessageContent, MessageSender as TelegramMessageSender, Update};
use tdlib::types::{self, Message as TelegramMessage};

use crate::config::{APP_ID, PROFILE};
use crate::session_manager::{ClientState, SessionManager};
use crate::tdlib::{Chat, ChatType};
use crate::utils::{spawn, MESSAGE_TRUNCATED_LENGTH};
use crate::Application;

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use gtk::gdk;

    use crate::session_manager::SessionManager;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/window.ui")]
    pub(crate) struct Window {
        pub(super) settings: gio::Settings,
        #[template_child]
        pub(super) session_manager: TemplateChild<SessionManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                session_manager: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.add_binding_action(
                gdk::Key::v,
                gdk::ModifierType::CONTROL_MASK,
                "win.paste",
                None,
            );
            klass.install_action("win.paste", None, move |widget, _, _| {
                widget.imp().session_manager.handle_paste_action();
            });

            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::SHIFT_MASK,
                "sidebar.begin-chats-search",
                None,
            );
            klass.install_action("sidebar.begin-chats-search", None, |widget, _, _| {
                widget.imp().session_manager.begin_chats_search();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Devel profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Load latest window state
            obj.load_window_size();

            // Start the thread that will receive tdlib's updates
            obj.start_tdlib_thread();

            // Set the online state of the active client based on
            // whether the window is active or not
            obj.connect_is_active_notify(|window| {
                spawn(clone!(@weak window => async move {
                    window
                        .imp()
                        .session_manager
                        .set_active_client_online(window.is_active()).await;
                }));
            });
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        // Save window state on delete event
        fn close_request(&self, obj: &Self::Type) -> gtk::Inhibit {
            // Close all clients. This must be blocking, otherwise the app might
            // close before the clients are properly closed, which is bad.
            self.session_manager.close_clients();

            if let Err(err) = obj.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            // Pass close request on to the parent
            self.parent_close_request(obj)
        }
    }

    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub(crate) struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl Window {
    pub(crate) fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create Window")
    }

    pub(crate) fn select_chat(&self, client_id: i32, chat_id: i64) {
        self.session_manager().select_chat(client_id, chat_id);
    }

    pub(crate) fn session_manager(&self) -> &SessionManager {
        &*self.imp().session_manager
    }

    fn start_tdlib_thread(&self) {
        let sender = self.create_update_channel();
        thread::spawn(move || loop {
            if let Some((update, client_id)) = tdlib::receive() {
                sender.send((update, client_id)).unwrap();
            }
        });
    }

    fn create_update_channel(&self) -> glib::Sender<(Update, i32)> {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return glib::Continue(false),
                move |(update, client_id)| {
                    obj.handle_update(update, client_id);
                    glib::Continue(true)
                }
            ),
        );
        sender
    }

    fn handle_update(&self, update: Update, client_id: i32) {
        match update {
            Update::NotificationGroup(update) => {
                self.add_notifications(update.added_notifications, client_id, update.chat_id);

                let app = self.application().unwrap();
                for notification_id in update.removed_notification_ids {
                    app.withdraw_notification(&notification_id.to_string());
                }
            }
            _ => self.imp().session_manager.handle_update(update, client_id),
        }
    }

    fn add_notifications(
        &self,
        notifications: Vec<types::Notification>,
        client_id: i32,
        chat_id: i64,
    ) {
        let client = self.imp().session_manager.client(client_id);
        if let Some(ref client) =
            client.filter(|client| matches!(client.state, ClientState::LoggedIn))
        {
            let app = self.application().unwrap();
            let chat = client.session.chat_list().get(chat_id);

            for notification in notifications {
                let notification_id = notification.id;
                let notification = match notification.r#type {
                    enums::NotificationType::NewMessage(data) => {
                        let mut title = chat.title();
                        let body = stringify_message_content(&data.message, &chat);

                        // Add the sender's name to the title if the chat is a group
                        if let ChatType::BasicGroup(_) | ChatType::Supergroup(_) = chat.type_() {
                            let sender_name = sender_name(&data.message.sender_id, &chat);
                            title.insert_str(0, &format!("{} – ", sender_name));
                        }

                        let notification = gio::Notification::new(&title);
                        notification.set_body(Some(&body));

                        notification.add_button_with_target_value(
                            &gettext("Mark as read"),
                            "app.mark-message-as-viewed",
                            Some(&(client_id, chat_id, data.message.id).to_variant()),
                        );
                        notification.add_button_with_target_value(
                            &gettext("Reply"),
                            "app.select-chat",
                            Some(&(client_id, chat_id).to_variant()),
                        );

                        Some(notification)
                    }
                    enums::NotificationType::NewCall(_) => {
                        let body = gettext("Incoming call");
                        let notification = gio::Notification::new(&chat.title());
                        notification.set_body(Some(&body));

                        Some(notification)
                    }
                    _ => None,
                };

                if let Some(notification) = notification {
                    notification.set_default_action_and_target_value(
                        "app.select-chat",
                        Some(&(client_id, chat_id).to_variant()),
                    );
                    app.send_notification(Some(&notification_id.to_string()), &notification);
                }
            }
        }
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();
        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        self.set_default_size(width, height);

        let is_maximized = imp.settings.boolean("is-maximized");
        if is_maximized {
            self.maximize();
        }
    }
}

fn sender_name(sender: &TelegramMessageSender, chat: &Chat) -> String {
    match sender {
        TelegramMessageSender::User(data) => {
            let user = chat.session().user_list().get(data.user_id);
            format!("{} {}", user.first_name(), user.last_name())
                .trim()
                .into()
        }
        TelegramMessageSender::Chat(data) => {
            let chat = chat.session().chat_list().get(data.chat_id);
            chat.title()
        }
    }
}

fn stringify_message_content(message: &TelegramMessage, chat: &Chat) -> String {
    match &message.content {
        MessageContent::MessageText(data) => data.text.text.clone(),
        MessageContent::MessageSticker(data) => {
            format!("{} {}", data.sticker.emoji, gettext("Sticker"))
        }
        MessageContent::MessagePhoto(data) => format!(
            "{}{}",
            gettext("Photo"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", data.caption.text)
            }
        ),
        MessageContent::MessageAudio(data) => format!(
            "{} - {}{}",
            data.audio.performer,
            data.audio.title,
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", data.caption.text)
            }
        ),
        MessageContent::MessageAnimation(data) => format!(
            "{}{}",
            gettext("GIF"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", data.caption.text)
            }
        ),
        MessageContent::MessageVideo(data) => format!(
            "{}{}",
            gettext("Video"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", data.caption.text)
            }
        ),
        MessageContent::MessageDocument(data) => format!(
            "{}{}",
            data.document.file_name,
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", data.caption.text)
            }
        ),
        MessageContent::MessageVoiceNote(data) => format!(
            "{}{}",
            gettext("Voice message"),
            if data.caption.text.is_empty() {
                String::new()
            } else {
                format!(", {}", data.caption.text)
            }
        ),
        MessageContent::MessageChatDeletePhoto => match chat.type_() {
            ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
                gettext("Channel photo removed")
            }
            _ => {
                let sender_name = sender_name(&message.sender_id, chat);
                gettext!("{} removed the group photo", sender_name)
            }
        },
        MessageContent::MessageChatChangePhoto(_) => match chat.type_() {
            ChatType::Supergroup(data) if data.is_channel() => gettext("Channel photo changed"),
            _ => {
                gettext!(
                    "{} changed group photo",
                    sender_name(&message.sender_id, chat),
                )
            }
        },
        MessageContent::MessagePinMessage(data) => {
            gettext!(
                "{} pinned {}",
                sender_name(&message.sender_id, chat),
                match chat.history().message_by_id(data.message_id) {
                    Some(data) => match data.content().0 {
                        MessageContent::MessageText(data) => {
                            let msg = data.text.text;
                            if msg.chars().count() > MESSAGE_TRUNCATED_LENGTH {
                                gettext!(
                                    "«{}…»",
                                    msg.chars()
                                        .take(MESSAGE_TRUNCATED_LENGTH - 1)
                                        .collect::<String>()
                                )
                            } else {
                                gettext!("«{}»", msg)
                            }
                        }
                        MessageContent::MessagePhoto(_) => gettext("a photo"),
                        MessageContent::MessageVideo(_) => gettext("a video"),
                        MessageContent::MessageSticker(data) => {
                            gettext!("a {} sticker", data.sticker.emoji)
                        }
                        MessageContent::MessageAnimation(_) => gettext("a GIF"),
                        MessageContent::MessageDocument(_) => gettext("a file"),
                        MessageContent::MessageAudio(_) => gettext("an audio file"),
                        _ => gettext("a message"),
                    },
                    None => gettext("a deleted message"),
                }
            )
        }
        MessageContent::MessageChatChangeTitle(data) => match chat.type_() {
            ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
                gettext!("Channel name was changed to «{}»", data.title)
            }
            _ => {
                gettext!(
                    "{} changed group name to «{}»",
                    sender_name(&message.sender_id, chat),
                    data.title
                )
            }
        },
        MessageContent::MessageChatJoinByLink => {
            gettext!(
                "{} joined the group via invite link",
                sender_name(&message.sender_id, chat)
            )
        }
        MessageContent::MessageChatJoinByRequest => {
            gettext!("{} joined the group", sender_name(&message.sender_id, chat))
        }
        MessageContent::MessageContactRegistered => {
            gettext!("{} joined Telegram", sender_name(&message.sender_id, chat))
        }
        _ => gettext("Unsupported message"),
    }
}
