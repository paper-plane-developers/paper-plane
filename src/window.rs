use gettextrs::gettext;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};
use std::thread;
use tdlib::enums::{self, Update};
use tdlib::types;

use crate::config::{APP_ID, PROFILE};
use crate::session_manager::{ClientState, SessionManager};
use crate::tdlib::{ChatType, Message};
use crate::utils::spawn;
use crate::{strings, Application};

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
            klass.bind_template();

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
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

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
        fn close_request(&self) -> gtk::Inhibit {
            // Close all clients. This must be blocking, otherwise the app might
            // close before the clients are properly closed, which is bad.
            self.session_manager.close_clients();

            if let Err(err) = self.obj().save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            // Pass close request on to the parent
            self.parent_close_request()
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
        glib::Object::builder().property("application", app).build()
    }

    pub(crate) fn select_chat(&self, client_id: i32, chat_id: i64) {
        self.session_manager().select_chat(client_id, chat_id);
    }

    pub(crate) fn session_manager(&self) -> &SessionManager {
        &self.imp().session_manager
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
                        let message = Message::new(data.message, &chat);
                        let mut body = strings::message_content(&message);

                        // Add the sender's name to the body if the chat is a group
                        if matches!(chat.type_(), ChatType::BasicGroup(_))
                            || matches!(chat.type_(), ChatType::Supergroup(s) if !s.is_channel())
                        {
                            let sender_name = strings::message_sender(message.sender());
                            body.insert_str(0, &(sender_name + ": "));
                        }

                        let notification = gio::Notification::new(&chat.title());
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
                    notification.set_default_action_and_target_value(
                        "app.select-chat",
                        Some(&(client_id, chat_id).to_variant()),
                    );

                    if let Some(avatar) = chat.avatar() {
                        let avatar_file = &avatar.0;
                        if avatar_file.local.is_downloading_completed {
                            if let Ok(texture) =
                                gdk::Texture::from_filename(&avatar_file.local.path)
                            {
                                notification.set_icon(&texture);
                            }
                            app.send_notification(
                                Some(&notification_id.to_string()),
                                &notification,
                            );
                        } else {
                            app.send_notification(
                                Some(&notification_id.to_string()),
                                &notification,
                            );
                            let (sender, receiver) = glib::MainContext::sync_channel::<
                                tdlib::types::File,
                            >(
                                Default::default(), 5
                            );
                            receiver.attach(
                                None,
                                clone!(@weak app => @default-return glib::Continue(false), move |file| {
                                    if file.local.is_downloading_completed {
                                        if let Ok(texture) = gdk::Texture::from_filename(&file.local.path) {
                                            notification.set_icon(&texture);
                                        }
                                        app.send_notification(Some(&notification_id.to_string()), &notification);
                                        glib::Continue(false)
                                    }
                                    else {
                                        glib::Continue(true)
                                    }
                                }));

                            client.session.download_file(avatar_file.id, sender);
                        }
                    }
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
