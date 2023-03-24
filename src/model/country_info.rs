use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::ops::Deref;

use gettextrs::gettext;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

#[derive(Clone, Debug, Default, glib::Boxed)]
#[boxed_type(name = "BoxedCallingCodes")]
pub(crate) struct CallingCodes(BTreeSet<String>);

impl CallingCodes {
    // Returns the first calling code in the set.
    pub(crate) fn first_or_empty(&self) -> &str {
        self.0.first().map(String::as_str).unwrap_or("")
    }
}

impl Deref for CallingCodes {
    type Target = BTreeSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::CountryInfo)]
    pub(crate) struct CountryInfo {
        #[property(get, set, construct_only)]
        pub(super) calling_codes: OnceCell<CallingCodes>,
        #[property(get, set, construct_only)]
        pub(super) country_code: OnceCell<String>,
        #[property(get, set, construct_only)]
        pub(super) name: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CountryInfo {
        const NAME: &'static str = "CountryInfo";
        type Type = super::CountryInfo;
    }

    impl ObjectImpl for CountryInfo {
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
    pub(crate) struct CountryInfo(ObjectSubclass<imp::CountryInfo>);
}

impl Default for CountryInfo {
    fn default() -> Self {
        Self::invalid()
    }
}

impl From<tdlib::types::CountryInfo> for CountryInfo {
    fn from(country_info: tdlib::types::CountryInfo) -> Self {
        Self::new(
            BTreeSet::from_iter(country_info.calling_codes),
            country_info.country_code,
            country_info.name,
        )
    }
}

impl CountryInfo {
    fn new(calling_codes: BTreeSet<String>, country_code: String, name: String) -> Self {
        glib::Object::builder()
            .property("calling-codes", CallingCodes(calling_codes))
            .property("country-code", country_code)
            .property("name", name)
            .build()
    }

    /// The invalid `CountryInfo` can be used when no valid calling code was specified by the user.
    pub(crate) fn invalid() -> Self {
        Self::new(
            Default::default(),
            "".into(),
            gettext("Invalid Country Code"),
        )
    }

    /// The test `CountryInfo` is used for Telegram test numbers and is set as `99966`.
    pub(crate) fn test() -> Self {
        Self::new(
            BTreeSet::from_iter(Some("99966".to_string())),
            "".into(),
            gettext("Test Account"),
        )
    }
}
