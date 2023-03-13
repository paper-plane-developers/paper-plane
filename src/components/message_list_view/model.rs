use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use std::cmp::Ordering;
use tdlib::enums::SearchMessagesFilter;
use thiserror::Error;

use super::{MessageListViewItem, MessageListViewItemType, MessageListViewType};
use crate::tdlib::{Chat, Message};

#[derive(Error, Debug)]
pub(crate) enum MessageListViewModelError {
    #[error("The chat history is already loading messages")]
    AlreadyLoading,
    #[error("TDLib error: {0:?}")]
    Tdlib(tdlib::types::Error),
}

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    #[derive(Debug, Default)]
    pub(crate) struct MessageListViewModel {
        pub(super) type_: Cell<MessageListViewType>,
        pub(super) chat: WeakRef<Chat>,
        pub(super) is_loading: Cell<bool>,
        pub(super) list: RefCell<VecDeque<MessageListViewItem>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageListViewModel {
        const NAME: &'static str = "MessageListViewModel";
        type Type = super::MessageListViewModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for MessageListViewModel {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<Chat>("chat")
                    .read_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for MessageListViewModel {
        fn item_type(&self) -> glib::Type {
            MessageListViewItem::static_type()
        }

        fn n_items(&self) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get(position as usize)
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageListViewModel(ObjectSubclass<imp::MessageListViewModel>)
        @implements gio::ListModel;
}

impl MessageListViewModel {
    pub(crate) fn new(type_: MessageListViewType, chat: &Chat) -> Self {
        let obj: MessageListViewModel = glib::Object::new();

        obj.imp().type_.set(type_);
        obj.imp().chat.set(Some(chat));

        chat.connect_new_message(clone!(@weak obj => move |_, message| {
            obj.push_front(message);
        }));
        chat.connect_deleted_message(clone!(@weak obj => move |_, message| {
            obj.remove(message);
        }));

        obj
    }

    /// Loads older messages from this chat history.
    ///
    /// Returns `true` when more messages can be loaded.
    pub(crate) async fn load_older_messages(
        &self,
        limit: i32,
    ) -> Result<bool, MessageListViewModelError> {
        let imp = self.imp();

        if imp.is_loading.get() {
            return Err(MessageListViewModelError::AlreadyLoading);
        }

        let oldest_message_id = imp
            .list
            .borrow()
            .iter()
            .rev()
            .find_map(|item| item.message())
            .map(|m| m.id())
            .unwrap_or_default();

        imp.is_loading.set(true);

        let result = match imp.type_.get() {
            MessageListViewType::ChatHistory => {
                self.chat().get_chat_history(oldest_message_id, limit).await
            }
            MessageListViewType::PinnedMessages => {
                self.chat()
                    .search_messages(
                        String::new(),
                        oldest_message_id,
                        limit,
                        Some(SearchMessagesFilter::Pinned),
                    )
                    .await
            }
        };

        imp.is_loading.set(false);

        let messages = result.map_err(MessageListViewModelError::Tdlib)?;

        if messages.is_empty() {
            return Ok(false);
        }

        self.append(messages);
        Ok(true)
    }

    fn items_changed(&self, position: u32, removed: u32, added: u32) {
        let imp = self.imp();

        // Insert day dividers where needed
        let added = {
            let position = position as usize;
            let added = added as usize;

            let mut list = imp.list.borrow_mut();
            let mut previous_timestamp = if position + 1 < list.len() {
                list.get(position + 1)
                    .and_then(|item| item.message_timestamp())
            } else {
                None
            };
            let mut dividers: Vec<(usize, MessageListViewItem)> = vec![];

            for (index, current) in list.range(position..position + added).enumerate().rev() {
                if let Some(current_timestamp) = current.message_timestamp() {
                    if Some(current_timestamp.ymd()) != previous_timestamp.as_ref().map(|t| t.ymd())
                    {
                        let divider_pos = position + index + 1;
                        dividers.push((
                            divider_pos,
                            MessageListViewItem::for_day_divider(current_timestamp.clone()),
                        ));
                        previous_timestamp = Some(current_timestamp);
                    }
                }
            }

            let dividers_len = dividers.len();
            for (position, item) in dividers {
                list.insert(position, item);
            }

            (added + dividers_len) as u32
        };

        // Check and remove no more needed day divider after removing messages
        let removed = {
            let mut removed = removed as usize;

            if removed > 0 {
                let mut list = imp.list.borrow_mut();
                let position = position as usize;
                let item_before_removed = list.get(position);

                if let Some(MessageListViewItemType::DayDivider(_)) =
                    item_before_removed.map(|i| i.type_())
                {
                    let item_after_removed = if position > 0 {
                        list.get(position - 1)
                    } else {
                        None
                    };

                    match item_after_removed.map(|item| item.type_()) {
                        None | Some(MessageListViewItemType::DayDivider(_)) => {
                            list.remove(position + removed);

                            removed += 1;
                        }
                        _ => {}
                    }
                }
            }

            removed as u32
        };

        // Check and remove no more needed day divider after adding messages
        let (position, removed) = {
            let mut removed = removed;
            let mut position = position as usize;

            if added > 0 && position > 0 {
                let mut list = imp.list.borrow_mut();
                let last_added_timestamp = list.get(position).unwrap().message_timestamp().unwrap();
                let next_item = list.get(position - 1);

                if let Some(MessageListViewItemType::DayDivider(date)) =
                    next_item.map(|item| item.type_())
                {
                    if date.ymd() == last_added_timestamp.ymd() {
                        list.remove(position - 1);

                        removed += 1;
                        position -= 1;
                    }
                }
            }

            (position as u32, removed)
        };

        self.upcast_ref::<gio::ListModel>()
            .items_changed(position, removed, added);
    }

    fn push_front(&self, message: Message) {
        self.imp()
            .list
            .borrow_mut()
            .push_front(MessageListViewItem::for_message(message));

        self.items_changed(0, 0, 1);
    }

    fn append(&self, messages: Vec<Message>) {
        let imp = self.imp();
        let added = messages.len();

        imp.list.borrow_mut().reserve(added);

        for message in messages {
            imp.list
                .borrow_mut()
                .push_back(MessageListViewItem::for_message(message));
        }

        let index = imp.list.borrow().len() - added;
        self.items_changed(index as u32, 0, added as u32);
    }

    fn remove(&self, message: Message) {
        let imp = self.imp();

        // Put this in a block, so that we only need to borrow the list once and the runtime
        // borrow checker does not panic in Self::items_changed when it borrows the list again.
        let index = {
            let mut list = imp.list.borrow_mut();

            // The elements in this list are ordered. While the day dividers are ordered
            // only by their date time, the messages are additionally sorted by their id. We
            // can exploit this by applying a binary search.
            let index = list
                .binary_search_by(|m| match m.type_() {
                    MessageListViewItemType::Message(other_message) => {
                        message.id().cmp(&other_message.id())
                    }
                    MessageListViewItemType::DayDivider(date_time) => {
                        let ordering = glib::DateTime::from_unix_utc(message.date() as i64)
                            .unwrap()
                            .cmp(date_time);
                        if let Ordering::Equal = ordering {
                            // We found the day divider of the message. Therefore, the message
                            // must be among the following elements.
                            Ordering::Greater
                        } else {
                            ordering
                        }
                    }
                })
                .unwrap();

            list.remove(index);
            index as u32
        };

        self.items_changed(index, 1, 0);
    }

    pub(crate) fn chat(&self) -> Chat {
        self.imp().chat.upgrade().unwrap()
    }
}
