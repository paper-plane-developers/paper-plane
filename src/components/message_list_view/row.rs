use adw::prelude::BinExt;
use adw::subclass::prelude::BinImpl;
use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::MessageContent;

use super::{MessageListViewEventRow, MessageListViewItem, MessageListViewItemType, MessageRow};
use crate::strings;
use crate::tdlib::SponsoredMessage;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub(crate) struct MessageListViewRow {
        /// An `ChatHistoryItem` or `SponsoredMessage`
        pub(super) item: RefCell<Option<glib::Object>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageListViewRow {
        const NAME: &'static str = "MessageListViewRow";
        type Type = super::MessageListViewRow;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for MessageListViewRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<glib::Object>("item")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.set_item(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.item().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageListViewRow {}
    impl BinImpl for MessageListViewRow {}
}

glib::wrapper! {
    pub(crate) struct MessageListViewRow(ObjectSubclass<imp::MessageListViewRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for MessageListViewRow {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageListViewRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().to_owned()
    }

    pub(crate) fn set_item(&self, item: Option<glib::Object>) {
        if self.item() == item {
            return;
        }

        if let Some(ref item) = item {
            if let Some(item) = item.downcast_ref::<MessageListViewItem>() {
                match item.type_() {
                    MessageListViewItemType::Message(message) => {
                        use tdlib::enums::MessageContent::*;

                        match message.content().0 {
                            MessageExpiredPhoto
                            | MessageExpiredVideo
                            | MessageCall(_)
                            | MessageBasicGroupChatCreate(_)
                            | MessageSupergroupChatCreate(_)
                            | MessageChatChangeTitle(_)
                            | MessageChatChangePhoto(_) // TODO: Show photo thumbnail
                            | MessageChatDeletePhoto
                            | MessageChatAddMembers(_)
                            | MessageChatJoinByLink
                            | MessageChatJoinByRequest
                            | MessageChatDeleteMember(_)
                            | MessagePinMessage(_)
                            | MessageScreenshotTaken
                            | MessageGameScore(_)
                            | MessageContactRegistered => {
                                self.get_or_create_event_row()
                                    .set_label(&strings::message_content(message));
                            }
                            _ => self.update_or_create_message_row(message.to_owned().upcast()),
                        }
                    }
                    MessageListViewItemType::DayDivider(date) => {
                        let fmt = if date.year() == glib::DateTime::now_local().unwrap().year() {
                            // Translators: This is a date format in the day divider without the year
                            gettext("%B %e")
                        } else {
                            // Translators: This is a date format in the day divider with the year
                            gettext("%B %e, %Y")
                        };
                        let date = date.format(&fmt).unwrap().to_string();

                        let child = self.get_or_create_event_row();
                        child.set_label(&date);
                    }
                }
            } else if let Some(sponsored_message) = item.downcast_ref::<SponsoredMessage>() {
                let content = &sponsored_message.content().0;
                if !matches!(content, MessageContent::MessageText(_)) {
                    log::warn!("Unexpected sponsored message of type: {:?}", content);
                }

                self.update_or_create_message_row(sponsored_message.to_owned().upcast());
            } else {
                unreachable!("Unexpected item type: {:?}", item);
            }
        }

        self.imp().item.replace(item);
        self.notify("item");
    }

    fn update_or_create_message_row(&self, message: glib::Object) {
        match self.child().and_then(|w| w.downcast::<MessageRow>().ok()) {
            Some(child) => child.set_message(message),
            None => {
                let child = MessageRow::new(&message);
                self.set_child(Some(&child));
            }
        }
    }

    fn get_or_create_event_row(&self) -> MessageListViewEventRow {
        if let Some(Ok(child)) = self
            .child()
            .map(|w| w.downcast::<MessageListViewEventRow>())
        {
            child
        } else {
            let child = MessageListViewEventRow::new();
            self.set_child(Some(&child));
            child
        }
    }
}
