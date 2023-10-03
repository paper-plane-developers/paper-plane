use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

use gio::prelude::*;
use glib::clone;
use gtk::gio;
use gtk::glib;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use thiserror::Error;

use crate::model;

#[derive(Error, Debug)]
pub(crate) enum ChatHistoryError {
    #[error("The chat history is already loading messages")]
    AlreadyLoading,
    #[error("TDLib error: {0:?}")]
    Tdlib(tdlib::types::Error),
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ChatHistoryModel {
        pub(super) chat: glib::WeakRef<model::Chat>,
        pub(super) is_loading: Cell<bool>,
        pub(super) list: RefCell<VecDeque<model::Message>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatHistoryModel {
        const NAME: &'static str = "ChatHistoryModel";
        type Type = super::ChatHistoryModel;
        type Interfaces = (gio::ListModel, gtk::SectionModel);
    }

    impl ObjectImpl for ChatHistoryModel {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<model::Chat>("chat")
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

    impl ListModelImpl for ChatHistoryModel {
        fn item_type(&self) -> glib::Type {
            model::Message::static_type()
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

    impl SectionModelImpl for ChatHistoryModel {
        fn section(&self, position: u32) -> (u32, u32) {
            let list = &*self.list.borrow();
            let message = list.get(position as usize).unwrap();

            let ymd = glib::DateTime::from_unix_local(message.date() as i64)
                .unwrap()
                .ymd();

            (
                if position == 0 {
                    0
                } else {
                    (0..position)
                        .rev()
                        .find(|i| {
                            ymd != glib::DateTime::from_unix_local(
                                list.get(*i as usize).unwrap().date() as i64,
                            )
                            .unwrap()
                            .ymd()
                        })
                        .map(|i| i + 1)
                        .unwrap_or(0)
                },
                (position + 1..list.len() as u32)
                    .find(|i| {
                        ymd != glib::DateTime::from_unix_local(
                            list.get(*i as usize).unwrap().date() as i64,
                        )
                        .unwrap()
                        .ymd()
                    })
                    .unwrap_or(list.len() as u32),
            )
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatHistoryModel(ObjectSubclass<imp::ChatHistoryModel>)
        @implements gio::ListModel;
}

impl ChatHistoryModel {
    pub(crate) fn new(chat: &model::Chat) -> Self {
        let obj: ChatHistoryModel = glib::Object::new();

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
    pub(crate) async fn load_older_messages(&self, limit: i32) -> Result<bool, ChatHistoryError> {
        let imp = self.imp();

        if imp.is_loading.get() {
            return Err(ChatHistoryError::AlreadyLoading);
        }

        let oldest_message_id = imp.list.borrow().back().map(|m| m.id()).unwrap_or_default();

        imp.is_loading.set(true);

        let result = self.chat().get_chat_history(oldest_message_id, limit).await;

        imp.is_loading.set(false);

        let messages = result.map_err(ChatHistoryError::Tdlib)?;

        if messages.is_empty() {
            return Ok(false);
        }

        self.append(messages);
        Ok(true)
    }

    fn push_front(&self, message: model::Message) {
        self.imp().list.borrow_mut().push_front(message);

        self.items_changed(0, 0, 1);
    }

    fn append(&self, messages: Vec<model::Message>) {
        let imp = self.imp();

        let added = messages.len();

        imp.list.borrow_mut().reserve(added);

        for message in messages {
            imp.list.borrow_mut().push_back(message);
        }

        let index = imp.list.borrow().len() - added;
        self.items_changed(index as u32, 0, added as u32);
    }

    fn remove(&self, message: model::Message) {
        let mut list = self.imp().list.borrow_mut();

        if let Ok(index) = list.binary_search_by(|m| message.id().cmp(&m.id())) {
            list.remove(index);

            drop(list);
            self.items_changed(index as u32, 1, 0);
        }
    }

    pub(crate) fn chat(&self) -> model::Chat {
        self.imp().chat.upgrade().unwrap()
    }
}
