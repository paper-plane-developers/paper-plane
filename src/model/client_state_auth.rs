use std::cell::RefCell;

use glib::prelude::Cast;
use glib::prelude::ObjectExt;
use glib::subclass::prelude::*;
use glib::Properties;
use gtk::glib;

use crate::model;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::ClientStateAuth)]
    pub(crate) struct ClientStateAuth {
        #[property(get, set, nullable, construct_only)]
        pub(super) client: glib::WeakRef<model::Client>,
        #[property(get)]
        pub(super) state: RefCell<Option<glib::Object>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ClientStateAuth {
        const NAME: &'static str = "ClientStateAuth";
        type Type = super::ClientStateAuth;
    }

    impl ObjectImpl for ClientStateAuth {
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
    pub(crate) struct ClientStateAuth(ObjectSubclass<imp::ClientStateAuth>);
}

impl From<&model::Client> for ClientStateAuth {
    fn from(client: &model::Client) -> Self {
        glib::Object::builder().property("client", client).build()
    }
}

impl ClientStateAuth {
    pub(crate) fn client_(&self) -> model::Client {
        self.client().unwrap()
    }

    fn set_state(&self, state: Option<glib::Object>) {
        if self.state() == state {
            return;
        }
        self.imp().state.replace(state);
        self.notify_state();
    }

    pub(crate) fn reset(&self) {
        self.set_state(Some(model::ClientStateAuthPhoneNumber::from(self).upcast()));
    }

    pub(crate) fn handle_update(&self, update: tdlib::types::UpdateAuthorizationState) {
        use tdlib::enums::AuthorizationState::*;

        let client = self.client_();
        if client.remove_if_auth() {
            utils::spawn(async move {
                _ = client.log_out().await;
            });
        }

        self.set_state(Some(match update.authorization_state {
            WaitPhoneNumber => model::ClientStateAuthPhoneNumber::from(self).upcast(),
            WaitCode(data) => match self
                .state()
                .and_then(|state| state.downcast::<model::ClientStateAuthCode>().ok())
            {
                Some(state) => {
                    state.set_data(model::BoxedAuthorizationStateWaitCode(data));
                    state
                }
                None => model::ClientStateAuthCode::new(self, data),
            }
            .upcast(),

            WaitOtherDeviceConfirmation(other_device) => match self
                .state()
                .and_then(|state| state.downcast::<model::ClientStateAuthOtherDevice>().ok())
            {
                Some(state) => {
                    state.set_data(model::BoxedAuthorizationStateWaitOtherDeviceConfirmation(
                        other_device,
                    ));
                    state
                }
                None => model::ClientStateAuthOtherDevice::new(self, other_device),
            }
            .upcast(),

            WaitPassword(password) => model::ClientStateAuthPassword::new(self, password).upcast(),
            WaitRegistration(registration) => {
                model::ClientStateAuthRegistration::new(self, registration).upcast()
            }
            other => unreachable!("Unsupported state during auth: {other:?}"),
        }));
    }
}
