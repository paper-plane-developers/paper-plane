use std::cell::OnceCell;
use std::sync::OnceLock;

use glib::prelude::*;
use glib::subclass::prelude::*;
use gtk::glib;

use crate::model;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "ChatHistoryItemType")]
pub(crate) enum ChatHistoryItemType {
    Message(model::Message),
    DayDivider(glib::DateTime),
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ChatHistoryItem {
        pub(super) type_: OnceCell<ChatHistoryItemType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatHistoryItem {
        const NAME: &'static str = "ChatHistoryItem";
        type Type = super::ChatHistoryItem;
    }

    impl ObjectImpl for ChatHistoryItem {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecBoxed::builder::<ChatHistoryItemType>("type")
                    .write_only()
                    .construct_only()
                    .build()]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "type" => {
                    let type_ = value.get::<ChatHistoryItemType>().unwrap();
                    self.type_.set(type_).unwrap();
                }
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatHistoryItem(ObjectSubclass<imp::ChatHistoryItem>);
}

impl ChatHistoryItem {
    pub(crate) fn for_message(message: model::Message) -> Self {
        let type_ = ChatHistoryItemType::Message(message);
        glib::Object::builder().property("type", type_).build()
    }

    pub(crate) fn for_day_divider(day: glib::DateTime) -> Self {
        let type_ = ChatHistoryItemType::DayDivider(day);
        glib::Object::builder().property("type", type_).build()
    }

    pub(crate) fn type_(&self) -> &ChatHistoryItemType {
        self.imp().type_.get().unwrap()
    }

    pub(crate) fn message(&self) -> Option<&model::Message> {
        if let ChatHistoryItemType::Message(message) = self.type_() {
            Some(message)
        } else {
            None
        }
    }

    pub(crate) fn message_timestamp(&self) -> Option<glib::DateTime> {
        if let ChatHistoryItemType::Message(message) = self.type_() {
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
