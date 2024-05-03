use std::cell::RefCell;
use std::sync::OnceLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;

use crate::model;
use crate::strings;
use crate::ui;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ChatHistoryRow {
        /// An `ChatHistoryItem` or `SponsoredMessage`
        pub(super) item: RefCell<Option<glib::Object>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatHistoryRow {
        const NAME: &'static str = "PaplChatHistoryRow";
        type Type = super::ChatHistoryRow;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for ChatHistoryRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecObject::builder::<glib::Object>("item")
                    .explicit_notify()
                    .build()]
            })
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

    impl WidgetImpl for ChatHistoryRow {
        fn map(&self) {
            self.parent_map();
            self.perform_history_action(ui::ChatHistory::add_to_viewed_message_ids);
        }

        fn unmap(&self) {
            self.parent_unmap();
            self.perform_history_action(ui::ChatHistory::remove_from_viewed_message_ids);
        }
    }
    impl BinImpl for ChatHistoryRow {}

    impl ChatHistoryRow {
        fn perform_history_action<F: Fn(&ui::ChatHistory, i64)>(&self, op: F) {
            let chat_history = utils::ancestor::<_, ui::ChatHistory>(&*self.obj());

            if let Some(item) = &*self.item.borrow() {
                if let Some(item) = item.downcast_ref::<model::ChatHistoryItem>() {
                    if let model::ChatHistoryItemType::Message(message) = item.type_() {
                        op(&chat_history, message.id());
                    }
                } else if let Some(message) = item.downcast_ref::<model::SponsoredMessage>() {
                    op(&chat_history, message.message_id());
                }
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatHistoryRow(ObjectSubclass<imp::ChatHistoryRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for ChatHistoryRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatHistoryRow {
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
            if let Some(item) = item.downcast_ref::<model::ChatHistoryItem>() {
                match item.type_() {
                    model::ChatHistoryItemType::Message(message) => {
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
                    model::ChatHistoryItemType::DayDivider(date) => {
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
            } else if let Some(sponsored_message) = item.downcast_ref::<model::SponsoredMessage>() {
                let content = &sponsored_message.content().0;
                if !matches!(content, tdlib::enums::MessageContent::MessageText(_)) {
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
        match self
            .child()
            .and_then(|w| w.downcast::<ui::MessageRow>().ok())
        {
            Some(child) => child.set_message(message),
            None => {
                let child = ui::MessageRow::new(&message);
                self.set_child(Some(&child));
            }
        }
    }

    fn get_or_create_event_row(&self) -> ui::EventRow {
        if let Some(Ok(child)) = self.child().map(|w| w.downcast::<ui::EventRow>()) {
            child
        } else {
            let child = ui::EventRow::new();
            self.set_child(Some(&child));
            child
        }
    }
}
