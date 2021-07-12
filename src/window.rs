use glib::{clone, SyncSender};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tdgrand::enums::{AuthorizationState, Update};
use tdgrand::functions;
use tokio::task;

use crate::config::{APP_ID, PROFILE};
use crate::Application;
use crate::Session;
use crate::RUNTIME;

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use gtk::{gio, CompositeTemplate};
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::Login;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/window.ui")]
    pub struct Window {
        pub settings: gio::Settings,
        pub receiver_handle: RefCell<Option<task::JoinHandle<()>>>,
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
        fn close_request(&self, obj: &Self::Type) -> gtk::Inhibit {
            self.receiver_should_stop.store(true, Ordering::Release);

            obj.close_clients();
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

    fn create_client(&self) {
        let client_id = tdgrand::create_client();

        let self_ = imp::Window::from_instance(self);
        self_.clients.borrow_mut().insert(client_id, None);
        self_.login.login_client(client_id);
        self_.main_stack.set_visible_child(&self_.login.get());

        // This call is important for login because TDLib requires the clients
        // to do at least a request to start receiving updates.
        RUNTIME.spawn(async move {
            functions::SetLogVerbosityLevel::new()
                .new_verbosity_level(2)
                .send(client_id)
                .await
                .unwrap();
        });
    }

    fn close_clients(&self) {
        let self_ = imp::Window::from_instance(self);

        for (client_id, _) in self_.clients.borrow().iter() {
            let client_id = *client_id;
            RUNTIME.spawn(async move {
                functions::Close::new().send(client_id).await.unwrap();
            });
        }
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
        RUNTIME.block_on(async move {
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

        if let Update::AuthorizationState(update) = update {
            if let AuthorizationState::Closed = update.authorization_state {
                let session = self_.clients.borrow_mut().remove(&client_id).unwrap();
                if let Some(session) = session {
                    self_.main_stack.remove(&session);
                }

                self.create_client();
            } else {
                self_
                    .login
                    .set_authorization_state(update.authorization_state);
            }
        } else if let Some(Some(session)) = self_.clients.borrow().get(&client_id) {
            session.handle_update(update);
        }
    }

    fn create_session(&self, client_id: i32) {
        let self_ = imp::Window::from_instance(self);
        let session = Session::new(client_id);

        self_.main_stack.add_child(&session);
        self_.main_stack.set_visible_child(&session);
        self_.clients.borrow_mut().insert(client_id, Some(session));
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
