use gtk::glib;
use gtk::glib::DateTime;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::tdlib::Message;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "ItemType")]
pub(crate) enum ItemType {
    Message(Message),
    DayDivider(DateTime),
}

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Debug, Default)]
    pub(crate) struct Item {
        pub(super) type_: OnceCell<ItemType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Item {
        const NAME: &'static str = "Item";
        type Type = super::Item;
    }

    impl ObjectImpl for Item {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoxed::new(
                    "type",
                    "Type",
                    "The type of this item",
                    ItemType::static_type(),
                    glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
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
                "type" => {
                    let type_ = value.get::<ItemType>().unwrap();
                    self.type_.set(type_).unwrap();
                }
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Item(ObjectSubclass<imp::Item>);
}

impl Item {
    pub(crate) fn for_message(message: Message) -> Self {
        let type_ = ItemType::Message(message);
        glib::Object::new(&[("type", &type_)]).expect("Failed to create Item")
    }

    pub(crate) fn for_day_divider(day: DateTime) -> Self {
        let type_ = ItemType::DayDivider(day);
        glib::Object::new(&[("type", &type_)]).expect("Failed to create Item")
    }

    pub(crate) fn type_(&self) -> &ItemType {
        self.imp().type_.get().unwrap()
    }

    pub(crate) fn message(&self) -> Option<&Message> {
        if let ItemType::Message(message) = self.type_() {
            Some(message)
        } else {
            None
        }
    }

    pub(crate) fn message_timestamp(&self) -> Option<DateTime> {
        if let ItemType::Message(message) = self.type_() {
            Some(
                glib::DateTime::from_unix_utc(message.date().into())
                    .and_then(|t| t.to_local())
                    .unwrap(),
            )
        } else {
            None
        }
    }
}
