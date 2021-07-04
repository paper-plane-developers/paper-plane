use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::enums::{self, Update};
use tdgrand::types::Message as TelegramMessage;

use crate::Session;
use crate::session::chat::History;

fn stringify_message(message: Option<TelegramMessage>) -> String {
    if let Some(message) = message {
        return match message.content {
            enums::MessageContent::MessageText(content) => content.text.text,
            _ => format!("<i>{}</i>", gettext("This message is unsupported")),
        }
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_string(
                        "last-message",
                        "Last Message",
                        "The last message sent on this chat",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_int64(
                        "order",
                        "Order",
                        "The parameter to determine the order of this chat in the chat list",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_int(
                        "unread-count",
                        "Unread Count",
                        "The unread messages count of this chat",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_string(
                        "draft-message",
                        "Draft Message",
                        "The draft message of this chat",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
            obj: &Self::Type,
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
                    obj.set_title(title);
                }
                "last-message" => {
                    let last_message = value.get().unwrap();
                    obj.set_last_message(last_message);
                }
                "order" => {
                    let order = value.get().unwrap();
                    obj.set_order(order);
                }
                "unread-count" => {
                    let unread_count = value.get().unwrap();
                    obj.set_unread_count(unread_count);
                }
                "draft-message" => {
                    let draft_message = value.get().unwrap();
                    obj.set_draft_message(draft_message);
                }
                "session" => {
                    let session = value.get().unwrap();
                    self.session.replace(session);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "title" => obj.title().to_value(),
                "last-message" => obj.last_message().to_value(),
                "order" => obj.order().to_value(),
                "unread-count" => obj.unread_count().to_value(),
                "draft-message" => obj.draft_message().to_value(),
                "history" => obj.history().to_value(),
                "session" => obj.session().to_value(),
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
        let priv_ = imp::Chat::from_instance(self);
        priv_.id.get()
    }

    pub fn title(&self) -> String {
        let priv_ = imp::Chat::from_instance(self);
        priv_.title.borrow().clone()
    }

    fn set_title(&self, title: String) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.title.replace(title);
        self.notify("title");
    }

    pub fn last_message(&self) -> String {
        let priv_ = imp::Chat::from_instance(self);
        priv_.last_message.borrow().clone()
    }

    fn set_last_message(&self, last_message: String) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.last_message.replace(last_message);
        self.notify("last-message");
    }

    pub fn order(&self) -> i64 {
        let priv_ = imp::Chat::from_instance(self);
        priv_.order.get()
    }

    fn set_order(&self, order: i64) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.order.set(order);
        self.notify("order");
    }

    pub fn unread_count(&self) -> i32 {
        let priv_ = imp::Chat::from_instance(self);
        priv_.unread_count.get()
    }

    fn set_unread_count(&self, unread_count: i32) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.unread_count.set(unread_count);
        self.notify("unread-count");
    }

    pub fn draft_message(&self) -> String {
        let priv_ = imp::Chat::from_instance(self);
        priv_.draft_message.borrow().clone()
    }

    fn set_draft_message(&self, draft_message: String) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.draft_message.replace(draft_message);
        self.notify("draft-message");
    }

    pub fn history(&self) -> &History {
        let priv_ = imp::Chat::from_instance(self);
        &priv_.history
    }

    pub fn session(&self) -> Option<Session> {
        let priv_ = imp::Chat::from_instance(self);
        priv_.session.borrow().to_owned()
    }

    pub fn connect_order_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("order"), f)
    }
}
