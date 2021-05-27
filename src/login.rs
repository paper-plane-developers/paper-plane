use crate::utils::do_async;
use crate::config;
use adw::NavigationDirection;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::{
    enums::AuthorizationState,
    functions,
    types::{PhoneNumberAuthenticationSettings, TdlibParameters},
};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/login.ui")]
    pub struct Login {
        pub client_id: Cell<i32>,
        #[template_child]
        pub next_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub next_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub next_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub content: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub phone_number_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub phone_number_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub code_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub code_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::PasswordEntry>,
        #[template_child]
        pub password_error_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("login.next", None, move |widget, _, _| widget.next());
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for Login {}
    impl BinImpl for Login {}
}

glib::wrapper! {
    pub struct Login(ObjectSubclass<imp::Login>)
        @extends gtk::Widget, adw::Bin;
}

impl Login {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Login")
    }

    pub fn set_client_id(&self, client_id: i32) {
        let priv_ = imp::Login::from_instance(self);
        priv_.client_id.set(client_id);
    }

    pub fn set_authorization_state(&self, state: AuthorizationState) {
        match state {
            AuthorizationState::WaitTdlibParameters => {
                self.send_tdlib_parameters();
            }
            AuthorizationState::WaitEncryptionKey(_) => {
                self.send_encryption_key();
            }
            AuthorizationState::WaitPhoneNumber => {
            }
            AuthorizationState::WaitCode(_) => {
                // Go to the next page
                let content = &imp::Login::from_instance(self).content;
                content.navigate(NavigationDirection::Forward);
            }
            AuthorizationState::WaitOtherDeviceConfirmation(_) => {
                todo!()
            }
            AuthorizationState::WaitRegistration(_) => {
                todo!()
            }
            AuthorizationState::WaitPassword(_) => {
                // Go to the next page
                let content = &imp::Login::from_instance(self).content;
                content.navigate(NavigationDirection::Forward);
            }
            AuthorizationState::Ready => {
                todo!()
            }
            AuthorizationState::LoggingOut => {
                todo!()
            }
            AuthorizationState::Closing => {
                todo!()
            }
            AuthorizationState::Closed => {
                todo!()
            }
        }
    }

    fn next(&self) {
        let content = &imp::Login::from_instance(self).content;
        let visible_page = content.visible_child_name().unwrap();

        self.freeze();

        if visible_page == "phone_number_page" {
            self.send_phone_number();
        } else if visible_page == "code_page" {
            self.send_code();
        } else if visible_page == "password_page" {
            self.send_password();
        }
    }

    fn freeze(&self) {
        let priv_ = imp::Login::from_instance(&self);

        self.action_set_enabled("login.next", false);
        priv_.next_stack
            .set_visible_child(&priv_.next_spinner.get());
        priv_.content.set_sensitive(false);
    }

    fn unfreeze(&self) {
        let priv_ = imp::Login::from_instance(&self);

        self.action_set_enabled("login.next", true);
        priv_.next_stack.set_visible_child(&priv_.next_label.get());
        priv_.content.set_sensitive(true);
    }

    fn send_tdlib_parameters(&self) {
        // TODO: make this parameters customizable
        let client_id = imp::Login::from_instance(self).client_id.get();
        let params = TdlibParameters {
            database_directory: "telegrand".to_string(),
            api_id: config::TG_API_ID,
            api_hash: config::TG_API_HASH.to_string(),
            system_language_code: "en-US".to_string(),
            device_model: "Desktop".to_string(),
            application_version: config::VERSION.to_string(),
            ..TdlibParameters::default()
        };
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::set_tdlib_parameters(client_id, params).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    let phone_number_error_label = &imp::Login::from_instance(&obj).phone_number_error_label;
                    phone_number_error_label.set_text(&err.message);
                    phone_number_error_label.set_visible(true);
                }
            }),
        );
    }

    fn send_encryption_key(&self) {
        // TODO: make the key customizable
        let client_id = imp::Login::from_instance(self).client_id.get();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::check_database_encryption_key(client_id, String::new()).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    let phone_number_error_label = &imp::Login::from_instance(&obj).phone_number_error_label;
                    phone_number_error_label.set_text(&err.message);
                    phone_number_error_label.set_visible(true);
                }
            }),
        );
    }

    fn send_phone_number(&self) {
        let priv_ = imp::Login::from_instance(self);
        let client_id = priv_.client_id.get();
        let phone_number = priv_.phone_number_entry.text().to_string();
        let settings = PhoneNumberAuthenticationSettings::default();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::set_authentication_phone_number(client_id, phone_number, settings).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    let phone_number_error_label = &imp::Login::from_instance(&obj).phone_number_error_label;
                    phone_number_error_label.set_text(&err.message);
                    phone_number_error_label.set_visible(true);
                }

                obj.unfreeze();
            }),
        );
    }

    fn send_code(&self) {
        let priv_ = imp::Login::from_instance(self);
        let client_id = priv_.client_id.get();
        let code = priv_.code_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::check_authentication_code(client_id, code).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    let code_error_label = &imp::Login::from_instance(&obj).code_error_label;
                    code_error_label.set_text(&err.message);
                    code_error_label.set_visible(true);
                }

                obj.unfreeze();
            }),
        );
    }

    fn send_password(&self) {
        let priv_ = imp::Login::from_instance(self);
        let client_id = priv_.client_id.get();
        let password = priv_.password_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::check_authentication_password(client_id, password).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    let password_error_label = &imp::Login::from_instance(&obj).password_error_label;
                    password_error_label.set_text(&err.message);
                    password_error_label.set_visible(true);
                }

                obj.unfreeze();
            }),
        );
    }
}
