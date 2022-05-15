use adw::prelude::ComboRowExt;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use locale_config::Locale;
use std::borrow::Cow;

use crate::tdlib::{CountryInfo, CountryList};

mod imp {
    use super::*;

    use adw::traits::ActionRowExt;
    use gettextrs::gettext;
    use gtk::CompositeTemplate;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/phone-number-input.ui")]
    pub(crate) struct PhoneNumberInput {
        /// The system's country code
        pub(super) system_country_code: OnceCell<Option<String>>,
        /// Tuple, that keeps track of the calling code's start and end position within the entered
        /// phone number. These bounds are used to replace and highlight the current calling code
        /// and to automatically select the number without calling code in case Telegram complains
        /// about an invalid number.
        pub(super) calling_code_bounds: Cell<(usize, usize)>,
        #[template_child]
        pub(super) list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub(super) combo_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PhoneNumberInput {
        const NAME: &'static str = "PhoneNumberInput";
        type Type = super::PhoneNumberInput;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Editable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PhoneNumberInput {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "model",
                        "Model",
                        "The model (CountryList) of this PhoneNumberInput",
                        CountryList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "number",
                        "Number",
                        "The current phone number of this PhoneNumberInput",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "model" => obj.set_model(value.get().unwrap()),
                "number" => {
                    obj.set_number(&value.get::<Option<String>>().unwrap().unwrap_or_default())
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "model" => obj.model().to_value(),
                "number" => obj.number().to_value(),
                _ => unimplemented!(),
            }
        }
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Set the system country code.
            self.system_country_code
                .set(
                    Locale::current()
                        .as_ref()
                        .split(',')
                        .next()
                        .and_then(|lang_country| lang_country.split('-').nth(1))
                        .map(str::to_string),
                )
                .unwrap();

            // Show the `CountryInfo`'s name in the combo box.
            self.combo_row
                .set_expression(Some(&CountryInfo::this_expression("name")));

            // We need these two handlers to block signals. Otherwise, the application would hang up
            // when the user enters a number or selects a country from the combo box.
            let combo_row_handler = Rc::new(RefCell::new(None));
            let entry_handler = Rc::new(RefCell::new(None));

            combo_row_handler.replace(Some(self.combo_row.connect_selected_item_notify(clone!(
                @weak obj,
                @strong entry_handler,
                => move |combo_row|
            {
                combo_row.set_subtitle("");

                let imp = obj.imp();

                let entry_handler = entry_handler.borrow();
                let entry_handler = entry_handler.as_ref().unwrap();

                let (pos_start, pos_end) = imp.calling_code_bounds.get();
                let number_prefix = (0..pos_start).map(|_| ' ')
                    .chain(Some('+'))
                    .collect::<String>();

                let selected_country_info = obj.selected_country_info().unwrap();
                let (number, pos_start, pos_end) = {
                    let first_calling_code = selected_country_info.calling_codes().first_or_empty();
                    (
                        [
                            number_prefix.as_str(),
                            first_calling_code,
                            obj.number()
                                .chars()
                                .skip(pos_end)
                                .collect::<String>()
                                .as_str(),
                        ]
                        .concat(),
                        pos_start,
                        pos_start + first_calling_code.chars().count() + 1,
                    )
                };

                imp.entry.block_signal(entry_handler);
                obj.set_number(&number);
                imp.entry.unblock_signal(entry_handler);

                imp.calling_code_bounds.set((pos_start, pos_end));

                obj.highlight_calling_code();
            }))));

            // Format the phone number and reset the cursor.
            entry_handler.replace(Some(self.entry.connect_changed(clone!(
                @weak obj,
                @strong combo_row_handler,
                => move |_|
            {
                if let Some(ref model) = obj.model() {
                    let imp = obj.imp();

                    let combo_row_handler = combo_row_handler.borrow();
                    let combo_row_handler = combo_row_handler.as_ref().unwrap();

                    let number = obj.number();
                    let (text_pos_start, text_pos_end, list_pos) = match number
                        .char_indices()
                        .find(|(_, c)| !c.is_whitespace())
                    {
                        Some((text_pos_start, c)) if c == '+' => {
                            let analysis = model.analyze_for_calling_code(
                                number
                                    .chars()
                                    .skip(text_pos_start as usize + 1)
                                    .collect::<String>()
                                    .trim_end(),
                                obj.preferred_country_code().as_deref(),
                            );
                            (
                                text_pos_start,
                                text_pos_start as u32 + analysis.code_len + 1,
                                analysis.list_pos,
                            )
                        }
                        _ => (0, 0, None),
                    };

                    imp.combo_row.block_signal(combo_row_handler);
                    match list_pos {
                        Some(list_pos) => {
                            imp.combo_row.set_selected(list_pos);
                            imp.combo_row.set_subtitle("");
                        }
                        None => {
                            imp.combo_row.set_selected(gtk::INVALID_LIST_POSITION);
                            imp.combo_row.set_subtitle(&gettext("You entered an invalid country code."));
                        }
                    }
                    imp.combo_row.unblock_signal(combo_row_handler);

                    imp.calling_code_bounds.set((text_pos_start as usize, text_pos_end as usize));

                    obj.highlight_calling_code();
                }
            }))));

            // We give focus the the phone number entry as soon as the user has selected an country
            // from the combo box.
            let focus_events = gtk::EventControllerFocus::new();
            self.combo_row.add_controller(&focus_events);
            focus_events.connect_leave(clone!(@weak obj => move |_| {
                // We need to set the cursor position at the end on the next idle.
                glib::idle_add_local(clone!(
                    @weak obj => @default-return glib::Continue(false), move || {
                        obj.imp().entry.set_position(i32::MAX);
                        glib::Continue(false)
                    }
                ));
            }));
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.list_box.unparent();
        }
    }

    impl WidgetImpl for PhoneNumberInput {
        fn grab_focus(&self, _: &Self::Type) -> bool {
            self.entry.grab_focus()
        }
    }

    impl EditableImpl for PhoneNumberInput {
        fn delegate(&self, _: &Self::Type) -> Option<gtk::Editable> {
            self.entry.delegate()
        }
    }
}

glib::wrapper! {
    pub(crate) struct PhoneNumberInput(ObjectSubclass<imp::PhoneNumberInput>)
        @extends gtk::Widget,
        @implements gtk::Editable, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PhoneNumberInput {
    pub(crate) fn model(&self) -> Option<CountryList> {
        self.imp()
            .combo_row
            .model()
            .map(|model| model.downcast::<CountryList>().unwrap())
    }

    pub(crate) fn set_model(&self, model: Option<&CountryList>) {
        if self.model().as_ref() == model {
            return;
        }
        self.imp().combo_row.set_model(model);
        self.notify("model");

        self.select_system_country_code();
    }

    pub(crate) fn number(&self) -> glib::GString {
        self.imp().entry.text()
    }

    pub(crate) fn set_number(&self, number: &str) {
        if self.number() == number {
            return;
        }
        self.imp().entry.set_text(number);
        self.notify("number");
    }

    /// Returns the currently selected `CountryInfo`.
    pub(crate) fn selected_country_info(&self) -> Option<CountryInfo> {
        self.imp()
            .combo_row
            .selected_item()
            .map(|item| item.downcast::<CountryInfo>().unwrap())
    }

    /// Performs a text selection of the whole number but leaves out the calling code.
    pub(crate) fn select_number_without_calling_code(&self) {
        let imp = self.imp();
        imp.entry
            .select_region(imp.calling_code_bounds.get().1 as i32, -1);
    }

    /// Function to determine the preferred country code. This is either the code of the currently
    /// selected country or the system country code.
    ///
    /// The functions prefers the code of the currently selected country over the system country
    /// code if both share the same calling code.
    fn preferred_country_code(&self) -> Option<Cow<str>> {
        let system_country_code = self.imp().system_country_code.get().unwrap().as_deref();

        self.model()
            .and_then(|model| {
                self.selected_country_info()
                    .as_ref()
                    .map(CountryInfo::country_code)
                    .and_then(|country_code_1| {
                        let has_same_calling_code = system_country_code
                            .map(|country_code_2| {
                                model.same_calling_code(&*country_code_1, country_code_2)
                            })
                            .unwrap_or_default();

                        if has_same_calling_code {
                            Some(Cow::Owned(country_code_1.to_owned()))
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| system_country_code.map(Cow::Borrowed))
    }

    /// Sets the selected item of the the combo box to that one with the system country code.
    fn select_system_country_code(&self) {
        if let Some(model) = self.model() {
            let imp = self.imp();

            let position = imp
                .system_country_code
                .get()
                .unwrap()
                .as_deref()
                .and_then(|country| model.country_code_pos(country))
                .unwrap_or(gtk::INVALID_LIST_POSITION);

            imp.combo_row.set_selected(position);
        }
    }

    /// Highlights the calling code from the rest of the number.
    fn highlight_calling_code(&self) {
        let imp = self.imp();

        let attr_list = pango::AttrList::new();

        let (pos_start, pos_end) = imp.calling_code_bounds.get();
        if pos_start < pos_end {
            let mut attr = pango::AttrInt::new_weight(pango::Weight::Bold);
            attr.set_start_index(pos_start as u32);
            attr.set_end_index(pos_end as u32);
            attr_list.insert(attr);

            let mut attr = pango::AttrInt::new_foreground_alpha(u16::MAX / 2);
            attr.set_start_index(pos_start as u32);
            attr.set_end_index(pos_end as u32);
            attr_list.insert(attr);
        }

        imp.entry.set_attributes(&attr_list);
    }
}
