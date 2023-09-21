use std::cell::RefCell;

use glib::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;
use gtk::glib;

use crate::model;

pub(crate) enum SendPhoneNumberResult {
    AlreadyLoggedIn(model::ClientStateSession),
    Err(tdlib::types::Error),
    Ok,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::ClientStateAuthPhoneNumber)]
    pub(crate) struct ClientStateAuthPhoneNumber {
        #[property(get, set)]
        pub(super) auth: glib::WeakRef<model::ClientStateAuth>,
        #[property(get, explicit_notify, nullable)]
        pub(super) country_list: RefCell<Option<model::CountryList>>,
        #[property(get, set)]
        pub(super) phone_number: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ClientStateAuthPhoneNumber {
        const NAME: &'static str = "ClientStateAuthPhoneNumber";
        type Type = super::ClientStateAuthPhoneNumber;
    }

    impl ObjectImpl for ClientStateAuthPhoneNumber {
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
    pub(crate) struct ClientStateAuthPhoneNumber(ObjectSubclass<imp::ClientStateAuthPhoneNumber>);
}

impl From<&model::ClientStateAuth> for ClientStateAuthPhoneNumber {
    fn from(auth: &model::ClientStateAuth) -> Self {
        glib::Object::builder().property("auth", auth).build()
    }
}

impl ClientStateAuthPhoneNumber {
    pub(crate) fn auth_(&self) -> model::ClientStateAuth {
        self.auth().unwrap()
    }

    pub(crate) async fn load_country_codes(&self) -> Result<(), tdlib::types::Error> {
        let client = self.auth_().client_();

        let use_test_dc = client.database_info().0.use_test_dc;

        match tdlib::functions::get_countries(client.id()).await {
            Ok(tdlib::enums::Countries::Countries(countries)) => {
                self.imp()
                    .country_list
                    .replace(Some(model::CountryList::from_td_object(
                        countries,
                        use_test_dc,
                    )));
                self.notify_country_list();

                Ok(())
            }
            Err(e) => {
                log::error!("Failed to load country codes: {e:?}");
                Err(e)
            }
        }
    }

    pub(crate) async fn send_phone_number(&self, phone_number: &str) -> SendPhoneNumberResult {
        let client = self.auth_().client_();

        // Check if we are already have an account logged in with that phone_number.
        let phone_number_digits = phone_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();

        self.set_phone_number(phone_number_digits.clone());

        let on_test_dc = client.database_info().0.use_test_dc;
        let sessions = client.client_manager_().sessions();
        let client_session = sessions.iter().find(|session| {
            on_test_dc == session.client_().database_info().0.use_test_dc
                && session.me_().phone_number().replace(' ', "") == phone_number_digits
        });

        match client_session {
            Some(client_session) => {
                // We just figured out that we already have an open session for that account.
                // Therefore we logout the client, with which we wanted to log in and delete its
                // just created database directory.
                SendPhoneNumberResult::AlreadyLoggedIn(client_session.to_owned())
            }
            None => {
                let result = tdlib::functions::set_authentication_phone_number(
                    phone_number.into(),
                    Some(tdlib::types::PhoneNumberAuthenticationSettings {
                        allow_flash_call: true,
                        allow_missed_call: true,
                        ..Default::default()
                    }),
                    client.id(),
                )
                .await;

                match result {
                    Ok(_) => SendPhoneNumberResult::Ok,
                    Err(e) => {
                        log::error!("Failed to use phone number: {e:?}");
                        SendPhoneNumberResult::Err(e)
                    }
                }
            }
        }
    }

    pub(crate) async fn request_qr_code(&self) -> Result<(), tdlib::types::Error> {
        let client = self.auth_().client_();
        let other_user_ids = client
            .client_manager_()
            .logged_in_users()
            .into_iter()
            .map(|user| user.id())
            .collect();

        match tdlib::functions::request_qr_code_authentication(other_user_ids, client.id()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to request QR code: {e:?}");
                Err(e)
            }
        }
    }
}
