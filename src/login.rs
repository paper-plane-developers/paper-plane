use crate::{RUNTIME, config};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::{
    enums::AuthorizationState,
    functions,
    types::TdlibParameters,
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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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
                todo!()
            }
            AuthorizationState::WaitCode(_) => {
                todo!()
            }
            AuthorizationState::WaitOtherDeviceConfirmation(_) => {
                todo!()
            }
            AuthorizationState::WaitRegistration(_) => {
                todo!()
            }
            AuthorizationState::WaitPassword(_) => {
                todo!()
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

        // TODO: handle errors
        RUNTIME.spawn(async move {
            functions::set_tdlib_parameters(client_id, params).await.unwrap();
        });
    }

    fn send_encryption_key(&self) {
        // TODO: make the key customizable and handle errors
        let client_id = imp::Login::from_instance(self).client_id.get();
        RUNTIME.spawn(async move {
            functions::check_database_encryption_key(client_id, String::new()).await.unwrap();
        });
    }
}
