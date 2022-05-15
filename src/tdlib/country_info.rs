use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::collections::BTreeSet;
use std::ops::Deref;
use tdlib::types;

#[derive(Clone, Debug, Default, glib::Boxed)]
#[boxed_type(name = "BoxedCallingCodes")]
pub(crate) struct CallingCodes(BTreeSet<String>);

impl CallingCodes {
    // Returns the first calling code in the set.
    pub(crate) fn first_or_empty(&self) -> &str {
        // TODO: Wait till self.first().unwrap() has stabilized.
        self.iter().next().map(String::as_str).unwrap_or("")
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
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;

    #[derive(Debug, Default)]
    pub(crate) struct CountryInfo {
        pub(super) calling_codes: OnceCell<CallingCodes>,
        pub(super) country_code: OnceCell<String>,
        pub(super) name: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CountryInfo {
        const NAME: &'static str = "CountryInfo";
        type Type = super::CountryInfo;
    }

    impl ObjectImpl for CountryInfo {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::new(
                        "calling-codes",
                        "Calling Codes",
                        "List of country calling codes",
                        CallingCodes::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecString::new(
                        "country-code",
                        "Country Code",
                        "A two-letter ISO 3166-1 alpha-2 country code",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecString::new(
                        "name",
                        "Name",
                        "Native name of the country",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "calling-codes" => self.calling_codes.set(value.get().unwrap()).unwrap(),
                "country-code" => self
                    .country_code
                    .set(value.get::<Option<String>>().unwrap().unwrap_or_default())
                    .unwrap(),
                "name" => self
                    .name
                    .set(value.get::<Option<String>>().unwrap().unwrap_or_default())
                    .unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "calling-codes" => obj.calling_codes().to_value(),
                "country-code" => obj.country_code().to_value(),
                "name" => obj.name().to_value(),
                _ => unimplemented!(),
            }
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

impl From<types::CountryInfo> for CountryInfo {
    fn from(country_info: types::CountryInfo) -> Self {
        Self::new(
            BTreeSet::from_iter(country_info.calling_codes),
            &country_info.country_code,
            &country_info.name,
        )
    }
}

impl CountryInfo {
    fn new(calling_codes: BTreeSet<String>, country_code: &str, name: &str) -> Self {
        glib::Object::new(&[
            ("calling-codes", &CallingCodes(calling_codes)),
            ("country-code", &country_code),
            ("name", &name),
        ])
        .expect("Failed to create CountryInfo")
    }

    /// The invalid `CountryInfo` can be used when no valid calling code was specified by the user.
    pub(crate) fn invalid() -> Self {
        Self::new(Default::default(), "", &gettext("Invalid Country Code"))
    }

    /// The test `CountryInfo` is used for Telegram test numbers and is set as `99966`.
    pub(crate) fn test() -> Self {
        Self::new(
            BTreeSet::from_iter(Some("99966".to_string())),
            "",
            &gettext("Test Account"),
        )
    }

    pub(crate) fn calling_codes(&self) -> &CallingCodes {
        self.imp().calling_codes.get().unwrap()
    }

    pub(crate) fn country_code(&self) -> &str {
        self.imp().country_code.get().unwrap()
    }

    pub(crate) fn name(&self) -> &str {
        self.imp().name.get().unwrap()
    }
}
