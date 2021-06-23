use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::enums::{self, Update};
use tdgrand::types::Chat as TelegramChat;
use tdgrand::types::Message as TelegramMessage;

fn stringify_message(message: Option<TelegramMessage>) -> Option<String> {
    if let Some(message) = message {
        return Some(match message.content {
            enums::MessageContent::MessageText(content) => content.text.text,
            _ => return None,
        })
    }

    None
}

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct Chat {
        pub title: RefCell<String>,
        pub last_message: RefCell<Option<String>>,
        pub order: Cell<i64>,
        pub unread_count: Cell<i32>,
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_int64(
                        "order",
                        "Order",
                        "The parameter to determine the order of this chat in the chat list",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_int(
                        "unread-count",
                        "Unread Count",
                        "The unread messages count of this chat",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => obj.title().to_value(),
                "last-message" => obj.last_message().to_value(),
                "order" => obj.order().to_value(),
                "unread-count" => obj.unread_count().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Chat(ObjectSubclass<imp::Chat>);
}

impl Chat {
    pub fn new(chat: TelegramChat) -> Self {
        let last_message = stringify_message(chat.last_message);

        let mut order = 0;
        for position in chat.positions {
            if let enums::ChatList::Main = position.list {
                order = position.order;
                break;
            }
        }

        glib::Object::new(&[
            ("title", &chat.title),
            ("last-message", &last_message),
            ("order", &order),
            ("unread-count", &chat.unread_count),
        ])
        .expect("Failed to create Chat")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::ChatTitle(update) => {
                self.set_title(update.title);
            },
            Update::ChatLastMessage(update) => {
                let message = stringify_message(update.last_message);
                self.set_last_message(message);

                for position in update.positions {
                    if let enums::ChatList::Main = position.list {
                        self.set_order(position.order);
                        break;
                    }
                }
            },
            Update::ChatPosition(update) => {
                if let enums::ChatList::Main = update.position.list {
                    self.set_order(update.position.order);
                }
            },
            Update::ChatReadInbox(update) => {
                self.set_unread_count(update.unread_count);
            },
            _ => (),
        }
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

    pub fn last_message(&self) -> Option<String> {
        let priv_ = imp::Chat::from_instance(self);
        priv_.last_message.borrow().clone()
    }

    fn set_last_message(&self, last_message: Option<String>) {
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

    pub fn connect_order_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("order"), f)
    }
}
