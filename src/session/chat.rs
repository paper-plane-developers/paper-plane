use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::{
    enums,
    types::Chat as TelegramChat,
    types::Message as TelegramMessage,
};

pub fn stringify_message(message: Option<TelegramMessage>) -> Option<String> {
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => obj.title().to_value(),
                "last-message" => obj.last_message().to_value(),
                "order" => obj.order().to_value(),
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
        ])
        .expect("Failed to create Chat")
    }

    pub fn set_title(&self, title: String) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.title.replace(title);
        self.notify("title");
    }

    pub fn set_last_message(&self, last_message: Option<String>) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.last_message.replace(last_message);
        self.notify("last-message");
    }

    fn title(&self) -> String {
        let priv_ = imp::Chat::from_instance(self);
        priv_.title.borrow().clone()
    }

    fn last_message(&self) -> Option<String> {
        let priv_ = imp::Chat::from_instance(self);
        priv_.last_message.borrow().clone()
    }

    fn set_order(&self, order: i64) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.order.set(order);
    }

    fn order(&self) -> i64 {
        let priv_ = imp::Chat::from_instance(self);
        priv_.order.get()
    }
}
