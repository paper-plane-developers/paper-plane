use crate::{Application, RUNTIME};
use crate::config::{APP_ID, PROFILE};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use rust_tdlib::client::Worker;

mod imp {
    use super::*;
    use crate::Login;
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use gtk::{CompositeTemplate, Inhibit, gio};
    use log::warn;
    use rust_tdlib::client::ConsoleAuthStateHandler;
    use rust_tdlib::client::client::ClientState;
    use rust_tdlib::client::tdlib_client::TdJson;
    use std::cell::RefCell;
    use tokio::task::JoinHandle;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/window.ui")]
    pub struct Window {
        #[template_child]
        pub login: TemplateChild<Login>,
        pub settings: gio::Settings,
        pub worker: RefCell<Option<Worker<ConsoleAuthStateHandler, TdJson>>>,
        pub waiter: RefCell<Option<JoinHandle<ClientState>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                login: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                worker: RefCell::default(),
                waiter: RefCell::default(),
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

            let builder =
                gtk::Builder::from_resource("/com/github/melix99/telegrand/shortcuts.ui");
            let shortcuts = builder.object("shortcuts").unwrap();
            obj.set_help_overlay(Some(&shortcuts));

            // Devel profile
            if PROFILE == "Devel" {
                obj.style_context().add_class("devel");
            }

            // Load latest window state
            obj.load_window_size();

            // Setup connection to telegram
            obj.setup_telegram();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        // Save window state on delete event
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            // Stop telegram connection
            obj.stop_telegram();

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

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
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

    fn setup_telegram(&self) {
        let mut worker = Worker::builder().build().unwrap();
        let _self = &imp::Window::from_instance(self);

        RUNTIME.block_on(async {
            let waiter = worker.start();
            _self.waiter.replace(Some(waiter));
        });

        _self.worker.replace(Some(worker));
    }

    fn stop_telegram(&self) {
        let _self = &imp::Window::from_instance(self);
        _self.worker.borrow().as_ref().unwrap().stop();

        RUNTIME.block_on(async {
            _self.waiter.borrow_mut().as_mut().unwrap().await.unwrap();
        });
    }
}
