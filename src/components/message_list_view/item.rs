use gtk::glib;
use gtk::glib::DateTime;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::tdlib::Message;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "MessageListViewItemType")]
pub(crate) enum MessageListViewItemType {
    Message(Message),
    DayDivider(DateTime),
}

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Debug, Default)]
    pub(crate) struct MessageListViewItem {
        pub(super) type_: OnceCell<MessageListViewItemType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageListViewItem {
        const NAME: &'static str = "MessageListViewItem";
        type Type = super::MessageListViewItem;
    }

    impl ObjectImpl for MessageListViewItem {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::builder::<MessageListViewItemType>("type")
                        .write_only()
                        .construct_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "type" => {
                    let type_ = value.get::<MessageListViewItemType>().unwrap();
                    self.type_.set(type_).unwrap();
                }
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageListViewItem(ObjectSubclass<imp::MessageListViewItem>);
}

impl MessageListViewItem {
    pub(crate) fn for_message(message: Message) -> Self {
        let type_ = MessageListViewItemType::Message(message);
        glib::Object::builder().property("type", type_).build()
    }

    pub(crate) fn for_day_divider(day: DateTime) -> Self {
        let type_ = MessageListViewItemType::DayDivider(day);
        glib::Object::builder().property("type", type_).build()
    }

    pub(crate) fn type_(&self) -> &MessageListViewItemType {
        self.imp().type_.get().unwrap()
    }

    pub(crate) fn message(&self) -> Option<&Message> {
        if let MessageListViewItemType::Message(message) = self.type_() {
            Some(message)
        } else {
            None
        }
    }

    pub(crate) fn message_timestamp(&self) -> Option<DateTime> {
        if let MessageListViewItemType::Message(message) = self.type_() {
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
