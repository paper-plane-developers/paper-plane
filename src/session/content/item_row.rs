use adw::{prelude::BinExt, subclass::prelude::BinImpl};
use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*};
use tdgrand::enums::MessageContent;

use crate::session::chat::{Item, ItemType};
use crate::session::content::message_row::{MessageSticker, MessageText};
use crate::session::content::{EventRow, MessageRowExt};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct ItemRow {
        pub item: RefCell<Option<Item>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemRow {
        const NAME: &'static str = "ContentItemRow";
        type Type = super::ItemRow;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for ItemRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "item",
                    "Item",
                    "The item represented by this row",
                    Item::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
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
                "item" => obj.set_item(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ItemRow {}
    impl BinImpl for ItemRow {}
}

glib::wrapper! {
    pub struct ItemRow(ObjectSubclass<imp::ItemRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for ItemRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ItemRow")
    }

    pub fn item(&self) -> Option<Item> {
        let self_ = imp::ItemRow::from_instance(self);
        self_.item.borrow().clone()
    }

    fn set_item(&self, item: Option<Item>) {
        if let Some(ref item) = item {
            match item.type_() {
                ItemType::Message(message) => match message.content().0 {
                    MessageContent::MessageSticker(data) if !data.sticker.is_animated => {
                        if let Some(Ok(child)) =
                            self.child().map(|w| w.downcast::<MessageSticker>())
                        {
                            child.set_message(Some(message.to_owned()));
                        } else {
                            let child = MessageSticker::new(message);
                            self.set_child(Some(&child));
                        }
                    }
                    _ => {
                        if let Some(Ok(child)) = self.child().map(|w| w.downcast::<MessageText>()) {
                            child.set_message(Some(message.to_owned()));
                        } else {
                            let child = MessageText::new(message);
                            self.set_child(Some(&child));
                        }
                    }
                },
                ItemType::DayDivider(date) => {
                    let fmt = if date.year() == glib::DateTime::new_now_local().unwrap().year() {
                        // Translators: This is a date format in the day divider without the year
                        gettext("%B %e")
                    } else {
                        // Translators: This is a date format in the day divider with the year
                        gettext("%B %e, %Y")
                    };
                    let date = date.format(&format!("<b>{}</b>", fmt)).unwrap().to_string();

                    let child = self.get_or_create_event_row();
                    child.set_label(&date);
                }
            }
        }

        let self_ = imp::ItemRow::from_instance(self);
        self_.item.replace(item);
    }

    fn get_or_create_event_row(&self) -> EventRow {
        if let Some(Ok(child)) = self.child().map(|w| w.downcast::<EventRow>()) {
            child
        } else {
            let child = EventRow::new();
            self.set_child(Some(&child));
            child
        }
    }
}
