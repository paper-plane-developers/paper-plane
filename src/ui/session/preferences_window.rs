use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::glib;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::ui;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/preferences_window.ui")]
    pub(crate) struct PreferencesWindow {
        pub(super) session: OnceCell<ui::Session>,
        #[template_child]
        pub(super) cache_size_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "PaplPreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async(
                "preferences.clear-cache",
                None,
                |widget, _, _| async move {
                    widget.clear_cache().await;
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<ui::Session>("session")
                    .construct_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "session" => self.session.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            utils::spawn(clone!(@weak obj => async move {
                obj.calculate_cache_size().await;
            }));
        }
    }

    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
}

glib::wrapper! {
    pub(crate) struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl PreferencesWindow {
    pub(crate) fn new(parent_window: Option<&gtk::Window>, session: &ui::Session) -> Self {
        glib::Object::builder()
            .property("transient-for", parent_window)
            .property(
                "application",
                parent_window.and_then(gtk::Window::application),
            )
            .property("session", session)
            .build()
    }

    async fn calculate_cache_size(&self) {
        let client_id = self.session().model().unwrap().client_().id();
        match tdlib::functions::get_storage_statistics(0, client_id).await {
            Ok(tdlib::enums::StorageStatistics::StorageStatistics(data)) => {
                let size = glib::format_size(data.size as u64);
                self.imp().cache_size_label.set_label(&size);
            }
            Err(e) => {
                log::warn!("Error getting the storage statistics: {e:?}");
            }
        }
    }

    async fn clear_cache(&self) {
        let client_id = self.session().model().unwrap().client_().id();
        match tdlib::functions::optimize_storage(
            0,
            0,
            0,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            false,
            0,
            client_id,
        )
        .await
        {
            Ok(tdlib::enums::StorageStatistics::StorageStatistics(data)) => {
                let size = glib::format_size(data.size as u64);
                self.imp().cache_size_label.set_label(&size);

                self.add_toast(adw::Toast::new(&gettext("Cache cleared")));
            }
            Err(e) => {
                log::warn!("Error optimizing the storage: {e:?}");
            }
        }
    }

    pub(crate) fn session(&self) -> &ui::Session {
        self.imp().session.get().unwrap()
    }
}
