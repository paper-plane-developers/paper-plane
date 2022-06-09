use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tdlib::enums::Update;
use tdlib::functions;
use tdlib::types::Chat as TelegramChat;

use crate::tdlib::Chat;
use crate::utils::spawn;
use crate::Session;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use glib::WeakRef;
    use indexmap::IndexMap;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct ChatList {
        pub(super) list: RefCell<IndexMap<i64, Chat>>,
        pub(super) unread_count: Cell<i32>,
        pub(super) session: WeakRef<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatList {
        const NAME: &'static str = "ChatList";
        type Type = super::ChatList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ChatList {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("positions-changed", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt::new(
                        "unread-count",
                        "Unread-Count",
                        "The unread count of this chat list",
                        0,
                        i32::MAX,
                        0,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "unread-count" => obj.unread_count().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for ChatList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Chat::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|(_, c)| c.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatList(ObjectSubclass<imp::ChatList>)
        @implements gio::ListModel;
}

impl ChatList {
    pub(crate) fn new(session: &Session) -> Self {
        let chat_list: ChatList = glib::Object::new(&[]).expect("Failed to create ChatList");
        chat_list.imp().session.set(Some(session));
        chat_list
    }

    pub(crate) fn fetch(&self, client_id: i32) {
        spawn(clone!(@weak self as obj => async move {
            let result = functions::load_chats(None, 20, client_id).await;

            if let Err(err) = result {
                // Error 404 means that all chats have been loaded
                if err.code != 404 {
                    log::error!("Received an error for LoadChats: {}", err.code);
                }
            } else {
                obj.fetch(client_id);
            }
        }));
    }

    pub(crate) fn handle_update(&self, update: Update) {
        use Update::*;

        match update {
            ChatAction(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatDraftMessage(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatIsBlocked(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatIsMarkedAsUnread(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatLastMessage(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatNotificationSettings(ref update_) => {
                self.handle_chat_update(update_.chat_id, update);
            }
            ChatPermissions(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatPhoto(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatPosition(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatReadInbox(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatReadOutbox(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatTitle(ref update_) => self.handle_chat_update(update_.chat_id, update),
            ChatUnreadMentionCount(ref update_) => self.handle_chat_update(update_.chat_id, update),
            DeleteMessages(ref update_) => self.handle_chat_update(update_.chat_id, update),
            MessageContent(ref update_) => self.handle_chat_update(update_.chat_id, update),
            MessageEdited(ref update_) => self.handle_chat_update(update_.chat_id, update),
            MessageMentionRead(ref update_) => self.handle_chat_update(update_.chat_id, update),
            MessageSendSucceeded(ref update_) => {
                self.handle_chat_update(update_.message.chat_id, update);
            }
            NewChat(update) => self.insert_chat(update.chat),
            NewMessage(ref update_) => self.handle_chat_update(update_.message.chat_id, update),
            UnreadMessageCount(ref update) => self.set_unread_count(update.unread_count),
            _ => {}
        }
    }

    fn handle_chat_update(&self, chat_id: i64, update: Update) {
        if let Some(chat) = self.imp().list.borrow().get(&chat_id) {
            chat.handle_update(update);
        }
    }

    /// Return the `Chat` of the specified `id`. Panics if the chat is not present.
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an `id` returned by TDLib, it should be expected that the
    /// relative `Chat` exists in the list.
    pub(crate) fn get(&self, id: i64) -> Chat {
        self.try_get(id).expect("Failed to get expected Chat")
    }

    pub(crate) fn try_get(&self, id: i64) -> Option<Chat> {
        self.imp().list.borrow().get(&id).cloned()
    }

    fn insert_chat(&self, chat: TelegramChat) {
        {
            let mut list = self.imp().list.borrow_mut();
            let chat_id = chat.id;
            let chat = Chat::new(chat, &self.session());

            chat.connect_order_notify(clone!(@weak self as obj => move |_, _| {
                obj.emit_by_name::<()>("positions-changed", &[]);
            }));

            list.insert(chat_id, chat);
        }

        self.item_added();
    }

    fn item_added(&self) {
        let list = self.imp().list.borrow();
        let position = list.len() - 1;
        self.items_changed(position as u32, 0, 1);
    }

    pub(crate) fn unread_count(&self) -> i32 {
        self.imp().unread_count.get()
    }

    fn set_unread_count(&self, unread_count: i32) {
        if self.unread_count() == unread_count {
            return;
        }

        self.imp().unread_count.set(unread_count);
        self.notify("unread-count");
    }

    pub(crate) fn session(&self) -> Session {
        self.imp().session.upgrade().unwrap()
    }

    pub(crate) fn connect_positions_changed<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("positions-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);

            None
        })
    }
}
