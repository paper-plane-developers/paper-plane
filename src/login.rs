use crate::utils::do_async;
use crate::config;
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
    use gtk::{CompositeTemplate, gio};
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/login.ui")]
    pub struct Login {
        pub client_id: Cell<i32>,
        #[template_child]
        pub previous_button: TemplateChild<gtk::Button>,
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
        pub welcome_page_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub custom_encryption_key_entry: TemplateChild<gtk::PasswordEntry>,
        #[template_child]
        pub use_test_dc_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub code_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub code_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::PasswordEntry>,
        #[template_child]
        pub password_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub encryption_key_entry: TemplateChild<gtk::PasswordEntry>,
        #[template_child]
        pub encryption_key_error_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("login.previous", None, move |widget, _, _| widget.previous());
            klass.install_action("login.next", None, move |widget, _, _| widget.next());
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
        fn constructed(&self, obj: &Self::Type) {
            obj.action_set_enabled("login.next", false);

            self.parent_constructed(obj);

            // Show the previous button on all pages except the
            // "phone number" page
            let priv_ = imp::Login::from_instance(obj);
            let previous_button = &*priv_.previous_button;
            priv_.content.connect_property_visible_child_name_notify(clone!(@weak previous_button => move |content| {
                let visible_page = content.visible_child_name().unwrap();
                if visible_page == "phone-number-page" {
                    previous_button.set_visible(false);
                } else {
                    previous_button.set_visible(true);
                }
            }));

            // Bind the use-test-dc setting to the relative switch
            let use_test_dc_switch = &*priv_.use_test_dc_switch;
            let settings = gio::Settings::new(config::APP_ID);
            settings
                .bind("use-test-dc", use_test_dc_switch, "state")
                .build();
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
                self.send_encryption_key(true);
            }
            AuthorizationState::WaitPhoneNumber => {
                self.action_set_enabled("login.next", true);
            }
            AuthorizationState::WaitCode(_) => {
                let content = &imp::Login::from_instance(self).content;
                content.set_visible_child_name("code-page");
            }
            AuthorizationState::WaitOtherDeviceConfirmation(_) => {
                todo!()
            }
            AuthorizationState::WaitRegistration(_) => {
                todo!()
            }
            AuthorizationState::WaitPassword(_) => {
                let content = &imp::Login::from_instance(self).content;
                content.set_visible_child_name("password-page");
            }
            AuthorizationState::Ready => {
                todo!()
            }
            AuthorizationState::LoggingOut => {
                todo!()
            }
            _ => ()
        }
    }

    fn previous(&self) {
        let content = &imp::Login::from_instance(self).content;
        content.set_visible_child_name("phone-number-page");
    }

    fn next(&self) {
        self.freeze();

        let priv_ = imp::Login::from_instance(self);
        let visible_page = priv_.content.visible_child_name().unwrap();
        if visible_page == "phone-number-page" {
            let encryption_key = priv_.custom_encryption_key_entry.text().to_string();
            if !encryption_key.is_empty() {
                self.change_encryption_key();
            }

            self.send_phone_number();
        } else if visible_page == "code-page" {
            self.send_code();
        } else if visible_page == "password-page" {
            self.send_password();
        } else if visible_page == "encryption-key-page" {
            self.send_encryption_key(false);
        }
    }

    fn freeze(&self) {
        self.action_set_enabled("login.previous", false);
        self.action_set_enabled("login.next", false);

        let priv_ = imp::Login::from_instance(self);
        priv_.next_stack.set_visible_child(&priv_.next_spinner.get());
        priv_.content.set_sensitive(false);
    }

    fn unfreeze(&self) {
        self.action_set_enabled("login.previous", true);
        self.action_set_enabled("login.next", true);

        let priv_ = imp::Login::from_instance(self);
        priv_.next_stack.set_visible_child(&priv_.next_label.get());
        priv_.content.set_sensitive(true);
    }

    fn send_tdlib_parameters(&self) {
        let priv_ = imp::Login::from_instance(self);
        let client_id = priv_.client_id.get();
        let use_test_dc = priv_.use_test_dc_switch.state();
        let params = TdlibParameters {
            use_test_dc: use_test_dc,
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
                    let welcome_page_error_label = &imp::Login::from_instance(&obj).welcome_page_error_label;
                    welcome_page_error_label.set_text(&err.message);
                    welcome_page_error_label.set_visible(true);
                }
            }),
        );
    }

    fn send_encryption_key(&self, use_empty_key: bool) {
        let priv_ = imp::Login::from_instance(self);
        let client_id = priv_.client_id.get();
        let encryption_key = {
            if use_empty_key {
                "".to_string()
            } else {
                priv_.encryption_key_entry.text().to_string()
            }
        };
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::check_database_encryption_key(client_id, encryption_key).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    let priv_ = imp::Login::from_instance(&obj);

                    // If we were trying an empty key, we now show the
                    // encryption key page to let the user input its key.
                    // Otherwise just show the error in the relative label.
                    if use_empty_key {
                        priv_.content.set_visible_child_name("encryption-key-page");
                        obj.action_set_enabled("login.next", true);
                    } else {
                        let encryption_key_error_label = &priv_.encryption_key_error_label;
                        encryption_key_error_label.set_text(&err.message);
                        encryption_key_error_label.set_visible(true);
                    }
                }

                if !use_empty_key {
                    obj.unfreeze();
                }
            }),
        );
    }

    fn change_encryption_key(&self) {
        let priv_ = imp::Login::from_instance(self);
        let client_id = priv_.client_id.get();
        let encryption_key = priv_.custom_encryption_key_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::set_database_encryption_key(client_id, encryption_key).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    let welcome_page_error_label = &imp::Login::from_instance(&obj).welcome_page_error_label;
                    welcome_page_error_label.set_text(&err.message);
                    welcome_page_error_label.set_visible(true);
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
                    let welcome_page_error_label = &imp::Login::from_instance(&obj).welcome_page_error_label;
                    welcome_page_error_label.set_text(&err.message);
                    welcome_page_error_label.set_visible(true);
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
