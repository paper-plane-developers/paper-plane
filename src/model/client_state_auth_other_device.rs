use std::cell::RefCell;

use glib::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;
use gtk::glib;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::ClientStateAuthOtherDevice)]
    pub(crate) struct ClientStateAuthOtherDevice {
        #[property(get, set, construct_only)]
        pub(super) auth: glib::WeakRef<model::ClientStateAuth>,
        #[property(get, set)]
        pub(super) data: RefCell<Option<model::BoxedAuthorizationStateWaitOtherDeviceConfirmation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ClientStateAuthOtherDevice {
        const NAME: &'static str = "ClientStateAuthOtherDevice";
        type Type = super::ClientStateAuthOtherDevice;
    }

    impl ObjectImpl for ClientStateAuthOtherDevice {
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
    pub(crate) struct ClientStateAuthOtherDevice(ObjectSubclass<imp::ClientStateAuthOtherDevice>);
}

impl ClientStateAuthOtherDevice {
    pub(crate) fn new(
        auth: &model::ClientStateAuth,
        data: model::AuthorizationStateWaitOtherDeviceConfirmation,
    ) -> Self {
        glib::Object::builder()
            .property("auth", auth)
            .property(
                "data",
                model::BoxedAuthorizationStateWaitOtherDeviceConfirmation(data),
            )
            .build()
    }

    pub(crate) fn auth_(&self) -> model::ClientStateAuth {
        self.auth().unwrap()
    }
}
