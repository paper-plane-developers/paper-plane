use std::cell::Cell;
use std::cell::RefCell;

use glib::prelude::ObjectExt;
use glib::subclass::prelude::*;
use glib::Properties;
use gtk::glib;

use crate::model;

pub(crate) enum SendPasswordRecoveryCodeResult {
    Expired,
    Err(tdlib::types::Error),
    Ok,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::ClientStateAuthPassword)]
    pub(crate) struct ClientStateAuthPassword {
        pub(super) password_recovery_ongoing: Cell<bool>,
        #[property(get, set, construct_only)]
        pub(super) auth: glib::WeakRef<model::ClientStateAuth>,
        #[property(get, set, construct_only)]
        pub(super) data: RefCell<Option<model::BoxedAuthorizationStateWaitPassword>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ClientStateAuthPassword {
        const NAME: &'static str = "ClientStateAuthPassword";
        type Type = super::ClientStateAuthPassword;
    }

    impl ObjectImpl for ClientStateAuthPassword {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub(crate) struct ClientStateAuthPassword(ObjectSubclass<imp::ClientStateAuthPassword>);
}

impl ClientStateAuthPassword {
    pub(crate) fn new(
        auth: &model::ClientStateAuth,
        password: tdlib::types::AuthorizationStateWaitPassword,
    ) -> Self {
        glib::Object::builder()
            .property("auth", auth)
            .property("data", model::BoxedAuthorizationStateWaitPassword(password))
            .build()
    }

    pub(crate) fn auth_(&self) -> model::ClientStateAuth {
        self.auth().unwrap()
    }

    pub(crate) async fn send_password(&self, password: String) -> Result<(), tdlib::types::Error> {
        let client = self.auth().unwrap().client().unwrap();
        match tdlib::functions::check_authentication_password(password, client.id()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to verify password: {e:?}");
                Err(e)
            }
        }
    }

    pub(crate) async fn recover_password(&self) -> Result<(), tdlib::types::Error> {
        let imp = self.imp();

        if imp.password_recovery_ongoing.get() {
            return Ok(());
        }

        let client = self.auth_().client().unwrap();

        let result = tdlib::functions::request_authentication_password_recovery(client.id()).await;
        if let Err(e) =
            tdlib::functions::request_authentication_password_recovery(client.id()).await
        {
            log::error!("Failed to recover password: {e:?}");
        }
        imp.password_recovery_ongoing.set(result.is_ok());

        result
    }

    pub(crate) async fn send_password_recovery_code(
        &self,
        recovery_code: String,
    ) -> SendPasswordRecoveryCodeResult {
        let client = self.auth_().client().unwrap();

        let result = tdlib::functions::recover_authentication_password(
            recovery_code,
            String::new(),
            String::new(),
            client.id(),
        )
        .await;

        match result {
            Ok(_) => SendPasswordRecoveryCodeResult::Ok,
            Err(e) => {
                if e.message == "PASSWORD_RECOVERY_EXPIRED" {
                    log::error!("Password revovery code is expired.");

                    // The same procedure is used as for the official client (as far as I
                    // understood from the code). Alternatively, we could send the user a new
                    // code, indicate that and stay on the recovery page.
                    self.imp().password_recovery_ongoing.set(false);

                    SendPasswordRecoveryCodeResult::Expired
                } else {
                    log::error!("Failed to send password code: {e:?}");
                    SendPasswordRecoveryCodeResult::Err(e)
                }
            }
        }
    }
}
