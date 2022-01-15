use gettextrs::gettext;
use gtk::glib::{clone, SyncSender};
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tdgrand::enums::{
    self, AuthorizationState, MessageContent, MessageSender as TelegramMessageSender, Update,
};
use tdgrand::types::{self, Message as TelegramMessage};
use tokio::task;

use crate::config::{APP_ID, PROFILE};
use crate::session::{Chat, ChatType};
use crate::session_manager::{ClientInfo, SessionManager};
use crate::Application;
use crate::RUNTIME;

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use gtk::gdk;
    use std::cell::RefCell;
    use std::sync::atomic::AtomicBool;

    use crate::session_manager::SessionManager;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/window.ui")]
    pub struct Window {
        pub settings: gio::Settings,
        pub receiver_handle: RefCell<Option<task::JoinHandle<()>>>,
        pub receiver_should_stop: Arc<AtomicBool>,

        #[template_child]
        pub session_manager: TemplateChild<SessionManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                receiver_handle: RefCell::default(),
                receiver_should_stop: Arc::default(),
                session_manager: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::SHIFT_MASK,
                "sidebar.begin-chats-search",
                None,
            );

            klass.install_action("sidebar.begin-chats-search", None, |widget, _, _| {
                Self::from_instance(widget)
                    .session_manager
                    .begin_chats_search();
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

            obj.start_receiver();

            // Set the online state of the active client based on
            // whether the window is active or not
            obj.connect_is_active_notify(|window| {
                Self::from_instance(window)
                    .session_manager
                    .set_active_client_online(window.is_active());
            });
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        // Save window state on delete event
        fn close_request(&self, obj: &Self::Type) -> gtk::Inhibit {
            self.receiver_should_stop.store(true, Ordering::Release);

            Self::from_instance(obj).session_manager.close_clients();
            obj.wait_receiver();

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
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create Window")
    }

    pub fn session_manager(&self) -> &SessionManager {
        &*imp::Window::from_instance(self).session_manager
    }

    fn start_receiver(&self) {
        let self_ = imp::Window::from_instance(self);
        let receiver_should_stop = self_.receiver_should_stop.clone();
        let sender = Arc::new(self.create_update_sender());
        let handle = RUNTIME.spawn(async move {
            loop {
                let receiver_should_stop = receiver_should_stop.clone();
                let sender = sender.clone();
                let stop = task::spawn_blocking(move || {
                    if let Some((update, client_id)) = tdgrand::receive() {
                        if receiver_should_stop.load(Ordering::Acquire) {
                            if let Update::AuthorizationState(ref update) = update {
                                if let AuthorizationState::Closed = update.authorization_state {
                                    return true;
                                }
                            }
                        }

                        sender.send((update, client_id)).unwrap();
                    }

                    false
                })
                .await
                .unwrap();

                if stop {
                    break;
                }
            }
        });

        self_.receiver_handle.replace(Some(handle));
    }

    fn wait_receiver(&self) {
        let self_ = imp::Window::from_instance(self);
        RUNTIME.block_on(async {
            self_
                .receiver_handle
                .borrow_mut()
                .as_mut()
                .unwrap()
                .await
                .unwrap();
        });
    }

    fn create_update_sender(&self) -> SyncSender<(Update, i32)> {
        let (sender, receiver) =
            glib::MainContext::sync_channel::<(Update, i32)>(Default::default(), 100);
        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return glib::Continue(false), move |(update, client_id)| {
                obj.handle_update(update, client_id);

                glib::Continue(true)
            }),
        );

        sender
    }

    fn handle_update(&self, update: Update, client_id: i32) {
        let self_ = imp::Window::from_instance(self);

        match update {
            Update::NotificationGroup(update) => {
                self.add_notifications(update.added_notifications, client_id, update.chat_id);

                let app = self.application().unwrap();
                for notification_id in update.removed_notification_ids {
                    app.withdraw_notification(&notification_id.to_string());
                }
            }
            _ => self_.session_manager.handle_update(update, client_id),
        }
    }

    fn add_notifications(
        &self,
        notifications: Vec<types::Notification>,
        client_id: i32,
        chat_id: i64,
    ) {
        let self_ = imp::Window::from_instance(self);

        if let Some(ClientInfo::LoggedIn(session)) = self_.session_manager.client_info(client_id) {
            let app = self.application().unwrap();
            let chat = session.chat_list().get(chat_id);

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
                    app.send_notification(Some(&notification_id.to_string()), &notification);
                }
            }
        }
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let self_ = imp::Window::from_instance(self);

        let (width, height) = self.default_size();
        self_.settings.set_int("window-width", width)?;
        self_.settings.set_int("window-height", height)?;

        self_
            .settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let self_ = imp::Window::from_instance(self);

        let width = self_.settings.int("window-width");
        let height = self_.settings.int("window-height");
        self.set_default_size(width, height);

        let is_maximized = self_.settings.boolean("is-maximized");
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
                if message.is_outgoing {
                    gettext("You removed the group photo")
                } else {
                    let sender_name = sender_name(&message.sender_id, chat);
                    gettext!("{} removed the group photo", sender_name)
                }
            }
        },
        MessageContent::MessageContactRegistered => {
            gettext!("{} joined Telegram", sender_name(&message.sender_id, chat))
        }
        _ => gettext("Unsupported message"),
    }
}
