use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use tdgrand::enums::{self, Update};
use tdgrand::functions;
use tdgrand::types::Message as TelegramMessage;

use crate::session::chat::{Item, ItemType, Message};
use crate::session::Chat;
use crate::utils::do_async;

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};
    use std::collections::{HashMap, VecDeque};

    #[derive(Debug, Default)]
    pub struct History {
        pub chat: WeakRef<Chat>,
        pub loading: Cell<bool>,
        pub list: RefCell<VecDeque<Item>>,
        pub message_map: RefCell<HashMap<i64, Message>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for History {
        const NAME: &'static str = "ChatHistory";
        type Type = super::History;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for History {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "chat",
                        "Chat",
                        "The chat relative to this history",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "loading",
                        "Loading",
                        "Whether the history is loading messages or not",
                        false,
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
                "chat" => self.chat.set(Some(&value.get().unwrap())),
                "loading" => obj.set_loading(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => obj.chat().to_value(),
                "loading" => obj.loading().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for History {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Item::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get(position as usize)
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct History(ObjectSubclass<imp::History>)
        @implements gio::ListModel;
}

impl History {
    pub fn new(chat: &Chat) -> Self {
        glib::Object::new(&[("chat", chat)]).expect("Failed to create History")
    }

    pub fn load_older_messages(&self) {
        if self.loading() {
            return;
        }

        let imp = self.imp();
        let chat = self.chat();
        let client_id = chat.session().client_id();
        let chat_id = chat.id();
        let oldest_message_id = imp
            .list
            .borrow()
            .iter()
            .find_map(|item| item.message())
            .map(|m| m.id())
            .unwrap_or_default();

        self.set_loading(true);

        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::GetChatHistory::new()
                    .chat_id(chat_id)
                    .from_message_id(oldest_message_id)
                    .limit(20)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Ok(enums::Messages::Messages(result)) = result {
                    if let Some(messages) = result.messages {
                        obj.prepend(messages);
                    }
                }

                obj.set_loading(false);
            }),
        );
    }

    pub fn message_by_id(&self, id: i64) -> Option<Message> {
        let imp = self.imp();
        imp.message_map.borrow().get(&id).cloned()
    }

    pub fn handle_update(&self, update: Update) {
        let imp = self.imp();

        match update {
            Update::NewMessage(update) => {
                self.append(update.message);
            }
            Update::MessageSendSucceeded(update) => {
                self.remove(update.old_message_id);
            }
            Update::MessageContent(ref update_) => {
                if let Some(message) = imp.message_map.borrow().get(&update_.message_id) {
                    message.handle_update(update);
                }
            }
            Update::DeleteMessages(update) => {
                if !update.from_cache {
                    for message_id in update.message_ids {
                        self.remove(message_id);
                    }
                }
            }
            _ => {}
        }
    }

    fn items_changed(&self, position: u32, removed: u32, added: u32) {
        let imp = self.imp();

        // Insert day dividers where needed
        let added = {
            let position = position as usize;
            let added = added as usize;

            let mut list = imp.list.borrow_mut();
            let mut previous_timestamp = if position > 0 {
                list.get(position - 1)
                    .and_then(|item| item.message_timestamp())
            } else {
                None
            };
            let mut dividers: Vec<(usize, Item)> = vec![];
            let mut index = position;

            for current in list.range(position..position + added) {
                if let Some(current_timestamp) = current.message_timestamp() {
                    if Some(current_timestamp.ymd()) != previous_timestamp.as_ref().map(|t| t.ymd())
                    {
                        dividers.push((index, Item::for_day_divider(current_timestamp.clone())));
                        previous_timestamp = Some(current_timestamp);
                        index += 1;
                    }
                }
                index += 1;
            }

            let dividers_len = dividers.len();
            for (position, item) in dividers {
                list.insert(position, item);
            }

            (added + dividers_len) as u32
        };

        // Check and remove no more needed day divider after removing messages
        let (position, removed) = {
            let mut position = position as usize;
            let mut removed = removed as usize;

            if removed > 0 {
                let mut list = imp.list.borrow_mut();
                let previous_item = if position > 0 {
                    list.get(position - 1)
                } else {
                    None
                };

                if let Some(ItemType::DayDivider(_)) = previous_item.map(|item| item.type_()) {
                    let item_after_removed = list.get(position + removed - 1);

                    match item_after_removed.map(|item| item.type_()) {
                        None | Some(ItemType::DayDivider(_)) => {
                            list.remove(position - 1);

                            position -= 1;
                            removed += 1;
                        }
                        _ => {}
                    }
                }
            }

            (position as u32, removed as u32)
        };

        // Check and remove no more needed day divider after adding messages
        let removed = {
            let mut removed = removed;

            if added > 0 {
                let position = position as usize;
                let added = added as usize;

                let mut list = imp.list.borrow_mut();
                let last_added_timestamp = list
                    .get(position + added - 1)
                    .unwrap()
                    .message_timestamp()
                    .unwrap();
                let next_item = list.get(position + added);

                if let Some(ItemType::DayDivider(date)) = next_item.map(|item| item.type_()) {
                    if date.ymd() == last_added_timestamp.ymd() {
                        list.remove(position + added);

                        removed += 1;
                    }
                }
            }

            removed
        };

        self.upcast_ref::<gio::ListModel>()
            .items_changed(position, removed, added);
    }

    pub fn append(&self, message: TelegramMessage) {
        let imp = self.imp();

        let mut message_map = imp.message_map.borrow_mut();

        if let Entry::Vacant(entry) = message_map.entry(message.id) {
            let message = Message::new(message, &self.chat());

            entry.insert(message.clone());

            imp.list.borrow_mut().push_back(Item::for_message(message));

            let index = imp.list.borrow().len() - 1;

            // We always need to drop all references before handing over control. Else, we could end
            // up with a borrowing error somewhere else.
            drop(message_map);
            self.items_changed(index as u32, 0, 1);
        }
    }

    fn prepend(&self, messages: Vec<TelegramMessage>) {
        let imp = self.imp();
        let chat = self.chat();
        let added = messages.len();

        imp.list.borrow_mut().reserve(added);

        for message in messages {
            let message = Message::new(message, &chat);

            imp.message_map
                .borrow_mut()
                .insert(message.id(), message.clone());

            imp.list.borrow_mut().push_front(Item::for_message(message));
        }

        self.items_changed(0, 0, added as u32);
    }

    fn remove(&self, message_id: i64) {
        let imp = self.imp();

        if let Some(message) = imp.message_map.borrow_mut().remove(&message_id) {
            // Put this in a block, so that we only need to borrow the list once and the runtime
            // borrow checker does not panic in Self::items_changed when it borrows the list again.
            let index = {
                let mut list = imp.list.borrow_mut();

                // The elements in this list are ordered. While the day dividers are ordered
                // only by their date time, the messages are additionally sorted by their id. We
                // can exploit this by applying a binary search.
                let index = list
                    .binary_search_by(|m| match m.type_() {
                        ItemType::Message(message) => message.id().cmp(&message_id),
                        ItemType::DayDivider(date_time) => {
                            let ordering = date_time.cmp(
                                &glib::DateTime::from_unix_utc(message.date() as i64).unwrap(),
                            );
                            if let Ordering::Equal = ordering {
                                // We found the day divider of the message. Therefore, the message
                                // must be among the following elements.
                                Ordering::Less
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
    }

    pub fn chat(&self) -> Chat {
        self.imp().chat.upgrade().unwrap()
    }

    pub fn set_loading(&self, loading: bool) {
        self.imp().loading.set(loading);
        self.notify("loading");
    }

    pub fn loading(&self) -> bool {
        self.imp().loading.get()
    }
}
