use std::cell::OnceCell;

use glib::prelude::ObjectExt;
use glib::subclass::prelude::*;
use glib::Properties;
use gtk::glib;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::ClientStateAuthRegistration)]
    pub(crate) struct ClientStateAuthRegistration {
        #[property(get, set, construct_only)]
        pub(super) auth: glib::WeakRef<model::ClientStateAuth>,
        #[property(get, set, construct_only)]
        pub(super) data: OnceCell<model::BoxedAuthorizationStateWaitRegistration>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ClientStateAuthRegistration {
        const NAME: &'static str = "ClientStateAuthRegistration";
        type Type = super::ClientStateAuthRegistration;
    }

    impl ObjectImpl for ClientStateAuthRegistration {
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
    pub(crate) struct ClientStateAuthRegistration(ObjectSubclass<imp::ClientStateAuthRegistration>);
}

impl ClientStateAuthRegistration {
    pub(crate) fn new(
        auth: &model::ClientStateAuth,
        data: tdlib::types::AuthorizationStateWaitRegistration,
    ) -> Self {
        glib::Object::builder()
            .property("auth", auth)
            .property("data", model::BoxedAuthorizationStateWaitRegistration(data))
            .build()
    }

    pub(crate) fn auth_(&self) -> model::ClientStateAuth {
        self.auth().unwrap()
    }

    pub(crate) async fn send_registration(
        &self,
        first_name: String,
        last_name: String,
    ) -> Result<(), tdlib::types::Error> {
        match tdlib::functions::register_user(first_name, last_name, self.auth_().client_().id())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to register account: {e:?}");
                Err(e)
            }
        }
    }
}
