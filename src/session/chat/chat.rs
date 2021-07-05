use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::enums::{self, Update};
use tdgrand::types::Message as TelegramMessage;

use crate::session::chat::History;
use crate::Session;

fn stringify_message(message: Option<TelegramMessage>) -> String {
    if let Some(message) = message {
        return match message.content {
            enums::MessageContent::MessageText(content) => content.text.text,
            _ => format!("<i>{}</i>", gettext("This message is unsupported")),
        };
    }
    String::new()
}

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct Chat {
        pub id: Cell<i64>,
        pub title: RefCell<String>,
        pub last_message: RefCell<String>,
        pub order: Cell<i64>,
        pub unread_count: Cell<i32>,
        pub draft_message: RefCell<String>,
        pub history: History,
        pub session: RefCell<Option<Session>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Chat {
        const NAME: &'static str = "Chat";
        type Type = super::Chat;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Chat {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int64(
                        "id",
                        "Id",
                        "The id of this chat",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "The title of this chat",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpec::new_string(
                        "last-message",
                        "Last Message",
                        "The last message sent on this chat",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_int64(
                        "order",
                        "Order",
                        "The parameter to determine the order of this chat in the chat list",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_int(
                        "unread-count",
                        "Unread Count",
                        "The unread messages count of this chat",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "draft-message",
                        "Draft Message",
                        "The draft message of this chat",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "history",
                        "History",
                        "The message history of this chat",
                        History::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
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
                "id" => {
                    let id = value.get().unwrap();
                    self.id.set(id);
                }
                "title" => {
                    let title = value.get().unwrap();
                    self.title.replace(title);
                }
                "last-message" => {
                    let last_message = value.get().unwrap();
                    self.last_message.replace(last_message);
                }
                "order" => {
                    let order = value.get().unwrap();
                    self.order.set(order);
                }
                "unread-count" => {
                    let unread_count = value.get().unwrap();
                    self.unread_count.set(unread_count);
                }
                "draft-message" => {
                    let draft_message = value.get().unwrap();
                    self.draft_message.replace(draft_message);
                }
                "session" => {
                    let session = value.get().unwrap();
                    self.session.replace(session);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "title" => self.title.borrow().to_value(),
                "last-message" => self.last_message.borrow().to_value(),
                "order" => self.order.get().to_value(),
                "unread-count" => self.unread_count.get().to_value(),
                "draft-message" => self.draft_message.borrow().to_value(),
                "history" => self.history.to_value(),
                "session" => self.session.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.bind_property("id", &self.history, "chat-id")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            obj.bind_property("session", &self.history, "session")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }
    }
}

glib::wrapper! {
    pub struct Chat(ObjectSubclass<imp::Chat>);
}

impl Chat {
    pub fn new(chat_id: i64, title: String) -> Self {
        glib::Object::new(&[("id", &chat_id), ("title", &title)])
            .expect("Failed to create Chat")
    }

    pub fn handle_update(&self, update: Update) {
        let priv_ = imp::Chat::from_instance(self);

        match update {
            Update::NewMessage(_) | Update::MessageContent(_) => {
                priv_.history.handle_update(update);
            }
            Update::ChatTitle(update) => {
                self.set_title(update.title);
            }
            Update::ChatLastMessage(update) => {
                let message = stringify_message(update.last_message);
                self.set_last_message(message);

                for position in update.positions {
                    if let enums::ChatList::Main = position.list {
                        self.set_order(position.order);
                        break;
                    }
                }
            }
            Update::ChatPosition(update) => {
                if let enums::ChatList::Main = update.position.list {
                    self.set_order(update.position.order);
                }
            }
            Update::ChatReadInbox(update) => {
                self.set_unread_count(update.unread_count);
            }
            Update::ChatDraftMessage(update) => {
                let mut draft_message = String::new();
                if let Some(message) = update.draft_message {
                    let content = message.input_message_text;
                    if let enums::InputMessageContent::InputMessageText(content) = content {
                        draft_message = content.text.text;
                    }
                }
                self.set_draft_message(draft_message);
            }
            _ => {}
        }
    }

    pub fn id(&self) -> i64 {
        self.property("id").unwrap().get().unwrap()
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }

    fn set_title(&self, title: String) {
        if self.title() != title {
            self.set_property("title", &title).unwrap();
        }
    }

    pub fn last_message(&self) -> String {
        self.property("last-message").unwrap().get().unwrap()
    }

    fn set_last_message(&self, last_message: String) {
        if self.last_message() != last_message {
            self.set_property("last-message", &last_message).unwrap();
        }
    }

    pub fn order(&self) -> i64 {
        self.property("order").unwrap().get().unwrap()
    }

    fn set_order(&self, order: i64) {
        if self.order() != order {
            self.set_property("order", &order).unwrap();
        }
    }

    pub fn connect_order_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("order"), f)
    }

    pub fn unread_count(&self) -> i32 {
        self.property("unread-count").unwrap().get().unwrap()
    }

    fn set_unread_count(&self, unread_count: i32) {
        if self.unread_count() != unread_count {
            self.set_property("unread-count", &unread_count).unwrap();
        }
    }

    pub fn draft_message(&self) -> String {
        self.property("draft-message").unwrap().get().unwrap()
    }

    fn set_draft_message(&self, draft_message: String) {
        if self.draft_message() != draft_message {
            self.set_property("draft-message", &draft_message).unwrap();
        }
    }

    pub fn history(&self) -> History {
        self.property("history").unwrap().get().unwrap()
    }

    pub fn session(&self) -> Option<Session> {
        self.property("session").unwrap().get().unwrap()
    }
}
