use glib::{clone, SyncSender};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task;
use tdgrand::enums::{AuthorizationState, Update};
use tdgrand::functions;

use crate::config::{APP_ID, PROFILE};
use crate::Application;
use crate::RUNTIME;
use crate::Session;

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use gtk::{gio, CompositeTemplate, Inhibit};
    use log::warn;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use tokio::task::JoinHandle;

    use crate::Login;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/window.ui")]
    pub struct Window {
        pub settings: gio::Settings,
        pub receiver_handle: RefCell<Option<JoinHandle<()>>>,
        pub receiver_should_stop: Arc<AtomicBool>,
        pub clients: RefCell<HashMap<i32, Option<Session>>>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub login: TemplateChild<Login>,
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
                clients: RefCell::default(),
                main_stack: TemplateChild::default(),
                login: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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

            self.login.connect_new_session(
                clone!(@weak obj => move |login| obj.create_session(login.client_id())),
            );

            obj.create_client();
            obj.start_receiver();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        // Save window state on delete event
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            self.receiver_should_stop.store(true, Ordering::Release);

            obj.close_clients();

            obj.wait_receiver();

            if let Err(err) = obj.save_window_size() {
                warn!("Failed to save window state, {}", &err);
            }

            Inhibit(false)
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
        glib::Object::new(&[("application", &app), ("icon-name", &APP_ID)])
            .expect("Failed to create Window")
    }

    fn create_client(&self) {
        let client_id = tdgrand::create_client();

        let priv_ = imp::Window::from_instance(self);
        priv_.clients.borrow_mut().insert(client_id, None);
        priv_.login.login_client(client_id);
        priv_.main_stack.set_visible_child(&priv_.login.get());

        // This call is important for login because TDLib requires the clients
        // to do at least a request to start receiving updates.
        RUNTIME.spawn(async move {
            functions::SetLogVerbosityLevel::new()
                .new_verbosity_level(2)
                .send(client_id).await.unwrap();
        });
    }

    fn close_clients(&self) {
        let priv_ = imp::Window::from_instance(self);

        for (client_id, _) in priv_.clients.borrow().iter() {
            let client_id = *client_id;
            RUNTIME.spawn(async move {
                functions::Close::new().send(client_id).await.unwrap();
            });
        }
    }

    fn start_receiver(&self) {
        let priv_ = imp::Window::from_instance(self);
        let receiver_should_stop = priv_.receiver_should_stop.clone();
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
                                    return true
                                }
                            }
                        }

                        sender.send((update, client_id)).unwrap();
                    }

                    false
                }).await.unwrap();

                if stop {
                    break;
                }
            }
        });

        priv_.receiver_handle.replace(Some(handle));
    }

    fn wait_receiver(&self) {
        let receiver_handle = &imp::Window::from_instance(self).receiver_handle;
        RUNTIME.block_on(async move {
            receiver_handle.borrow_mut().as_mut().unwrap().await.unwrap();
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
        let priv_ = imp::Window::from_instance(self);

        if let Update::AuthorizationState(update) = update {
            if let AuthorizationState::Closed = update.authorization_state {
                let session = priv_.clients.borrow_mut().remove(&client_id).unwrap();
                if let Some(session) = session {
                    priv_.main_stack.remove(&session);
                }

                self.create_client();
            } else {
                priv_.login.set_authorization_state(update.authorization_state);
            }
        } else if let Some(Some(session)) = priv_.clients.borrow().get(&client_id) {
            session.handle_update(update);
        }
    }

    fn create_session(&self, client_id: i32) {
        let priv_ = imp::Window::from_instance(self);
        let session = Session::new(client_id);

        priv_.main_stack.add_child(&session);
        priv_.main_stack.set_visible_child(&session);
        priv_.clients.borrow_mut().insert(client_id, Some(session));
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = &imp::Window::from_instance(self).settings;

        let size = self.default_size();
        settings.set_int("window-width", size.0)?;
        settings.set_int("window-height", size.1)?;

        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = &imp::Window::from_instance(self).settings;

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        self.set_default_size(width, height);

        let is_maximized = settings.boolean("is-maximized");
        if is_maximized {
            self.maximize();
        }
    }
}
