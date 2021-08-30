use gtk::{glib, glib::DateTime, prelude::*, subclass::prelude::*};

use crate::session::chat::Message;

#[derive(Debug, Clone)]
pub enum ItemType {
    Message(Message),
    DayDivider(DateTime),
}

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "BoxedItemType")]
pub struct BoxedItemType(ItemType);

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Debug, Default)]
    pub struct Item {
        pub type_: OnceCell<ItemType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Item {
        const NAME: &'static str = "Item";
        type Type = super::Item;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Item {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boxed(
                    "type",
                    "Type",
                    "The type of this item",
                    BoxedItemType::static_type(),
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
                    let type_ = value.get::<BoxedItemType>().unwrap();
                    self.type_.set(type_.0).unwrap();
                }
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Item(ObjectSubclass<imp::Item>);
}

impl Item {
    pub fn for_message(message: Message) -> Self {
        let type_ = BoxedItemType(ItemType::Message(message));
        glib::Object::new(&[("type", &type_)]).expect("Failed to create Item")
    }

    pub fn for_day_divider(day: DateTime) -> Self {
        let type_ = BoxedItemType(ItemType::DayDivider(day));
        glib::Object::new(&[("type", &type_)]).expect("Failed to create Item")
    }

    pub fn type_(&self) -> &ItemType {
        let self_ = imp::Item::from_instance(self);
        self_.type_.get().unwrap()
    }

    pub fn message(&self) -> Option<&Message> {
        if let ItemType::Message(message) = self.type_() {
            Some(message)
        } else {
            None
        }
    }

    pub fn message_timestamp(&self) -> Option<DateTime> {
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
