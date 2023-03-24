use std::cell::OnceCell;
use std::iter::FromIterator;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use indexmap::IndexMap;
use tdlib::types;

use crate::model::CountryInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct CountryList(pub(super) OnceCell<IndexMap<String, CountryInfo>>);

    #[glib::object_subclass]
    impl ObjectSubclass for CountryList {
        const NAME: &'static str = "CountryList";
        type Type = super::CountryList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for CountryList {}

    impl ListModelImpl for CountryList {
        fn item_type(&self) -> glib::Type {
            CountryInfo::static_type()
        }

        fn n_items(&self) -> u32 {
            self.obj().list().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.obj()
                .list()
                .get_index(position as usize)
                .map(|(_, i)| i.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    /// A List of `CountryInfo`s sorted by their country codes.
    pub(crate) struct CountryList(ObjectSubclass<imp::CountryList>) @implements gio::ListModel;
}

impl CountryList {
    pub(crate) fn from_td_object(mut countries: types::Countries, use_test_dc: bool) -> Self {
        let obj: Self = glib::Object::new();

        // This has to be sorted here directly, as it seems the AdwComboBoxRow (where the Country-
        // List is used) can't be controlled through an `SingleSelection` and `SortListModel`.
        countries.countries.sort_by(|a, b| a.name.cmp(&b.name));

        obj.imp()
            .0
            .set(IndexMap::from_iter(
                countries
                    .countries
                    .into_iter()
                    .map(|country_info| {
                        (
                            country_info.country_code.clone(),
                            CountryInfo::from(country_info),
                        )
                    })
                    .chain(if use_test_dc {
                        Some((String::new(), CountryInfo::test()))
                    } else {
                        None
                    }),
            ))
            .unwrap();

        obj
    }

    /// Returns the list position of the item with the specified country code or `None` if no item
    /// could be found.
    pub(crate) fn country_code_pos(&self, country_code: &str) -> Option<u32> {
        self.list()
            .get_full(country_code)
            .map(|(position, ..)| position as u32)
    }

    /// Returns the list position of the item with the specified calling code or `None` if no item
    /// could be found.
    ///
    /// Different countries may share the same calling code. So, an optional country code can be
    /// specified. In this way, the item with that country code is preferred.
    pub(crate) fn calling_code_pos(
        &self,
        calling_code: &str,
        country_code: Option<&str>,
    ) -> Option<u32> {
        let list = self.list();

        country_code
            .and_then(|country_code| {
                self.country_code_pos(country_code).and_then(|position| {
                    if list[position as usize]
                        .calling_codes()
                        .contains(calling_code)
                    {
                        Some(position)
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                list.iter()
                    .enumerate()
                    .find(|(_, (_, info))| info.calling_codes().contains(calling_code))
                    .map(|(position, _)| position as u32)
            })
    }

    /// Analyzes the the specified text for a calling code and returns a `TextAnalysis` result.
    ///
    /// The passed text has to be start trimmed in order to have the chance of finding a calling
    /// code.
    ///
    /// An optional country code can be specified. In this way, the calling code with that country
    /// code is preferred.
    pub(crate) fn analyze_for_calling_code(
        &self,
        text: &str,
        country_code: Option<&str>,
    ) -> CallingCodeAnalysis {
        if text.is_empty() {
            CallingCodeAnalysis {
                code_len: 0,
                list_pos: None,
            }
        } else {
            self.calling_code_pos(&text.replace(' ', ""), country_code)
                .map(|position| CallingCodeAnalysis {
                    code_len: text.trim_end().chars().count() as u32,
                    list_pos: Some(position),
                })
                .unwrap_or_else(|| {
                    let shortened = text
                        .chars()
                        .take(text.chars().count() - 1)
                        .collect::<String>();
                    self.analyze_for_calling_code(&shortened, country_code)
                })
        }
    }

    /// Checks whether both specified country codes share a same calling code.
    pub(crate) fn same_calling_code(&self, country_code_1: &str, country_code_2: &str) -> bool {
        self.country_code_pos(country_code_1)
            .and_then(|position_1| {
                self.country_code_pos(country_code_2).map(|position_2| {
                    let list = self.list();

                    list[position_1 as usize]
                        .calling_codes()
                        .intersection(&list[position_2 as usize].calling_codes())
                        .next()
                        .is_some()
                })
            })
            .unwrap_or_default()
    }

    fn list(&self) -> &IndexMap<String, CountryInfo> {
        self.imp().0.get().unwrap()
    }
}

/// The result of a calling code analysis.
pub(crate) struct CallingCodeAnalysis {
    /// The length in chars of the found calling code.
    /// The length can be translated to the position in the analyzed text as the calling code in the
    /// analyzed text starts at position 0.
    /// The value will be `0` if no calling code was found.
    pub(crate) code_len: u32,
    /// The list position of the associated `CountryInfo`. Is `None` if no valid item could be
    /// found.
    pub(crate) list_pos: Option<u32>,
}
