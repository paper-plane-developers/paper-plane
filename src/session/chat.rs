use crate::utils::do_async;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::{
    enums,
    functions,
    types::Chat as TelegramChat,
    types::Message as TelegramMessage,
};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct Chat {
        pub chat_id: Cell<i64>,
        pub client_id: Cell<i32>,
        pub title: RefCell<String>,
        pub last_message: RefCell<Option<String>>,
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
                        "chat-id",
                        "Chat Id",
                        "The id of this chat",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_int(
                        "client-id",
                        "Client Id",
                        "The client id",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "The title of this chat",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "last-message",
                        "Last Message",
                        "The last message sent on this chat",
                        None,
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
                "chat-id" => {
                    let chat_id = value.get().unwrap();
                    self.chat_id.set(chat_id);
                }
                "client-id" => {
                    let client_id = value.get().unwrap();
                    self.client_id.set(client_id);
                }
                "title" => {
                    let title = value.get().unwrap();
                    obj.set_title(title);
                }
                "last-message" => {
                    let last_message = value.get().unwrap();
                    obj.set_last_message(last_message);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat-id" => obj.chat_id().to_value(),
                "client-id" => obj.client_id().to_value(),
                "title" => obj.title().to_value(),
                "last-message" => obj.last_message().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.load_chat();
        }
    }
}

glib::wrapper! {
    pub struct Chat(ObjectSubclass<imp::Chat>);
}

impl Chat {
    pub fn new(chat_id: i64, client_id: i32) -> Self {
        glib::Object::new(&[("chat-id", &chat_id), ("client-id", &client_id)])
            .expect("Failed to create Chat")
    }

    fn chat_id(&self) -> i64 {
        let priv_ = imp::Chat::from_instance(self);
        priv_.chat_id.get()
    }

    fn client_id(&self) -> i32 {
        let priv_ = imp::Chat::from_instance(self);
        priv_.client_id.get()
    }

    fn title(&self) -> String {
        let priv_ = imp::Chat::from_instance(self);
        priv_.title.borrow().clone()
    }

    fn set_title(&self, title: String) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.title.replace(title);
    }

    fn last_message(&self) -> Option<String> {
        let priv_ = imp::Chat::from_instance(self);
        priv_.last_message.borrow().clone()
    }

    fn set_last_message(&self, last_message: Option<String>) {
        let priv_ = imp::Chat::from_instance(self);
        priv_.last_message.replace(last_message);
    }

    fn stringify_message(&self, message: Option<TelegramMessage>) -> Option<String> {
        if let Some(message) = message {
            return Some(match message.content {
                enums::MessageContent::MessageText(content) => content.text.text,
                _ => return None,
            })
        }

        None
    }

    fn set_telegram_chat(&self, telegram_chat: TelegramChat) {
        self.set_property("title", telegram_chat.title).unwrap();

        let last_message = self.stringify_message(telegram_chat.last_message);
        self.set_property("last-message", last_message).unwrap();
    }

    fn load_chat(&self) {
        let client_id = self.client_id();
        let chat_id = self.chat_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::get_chat(client_id, chat_id).await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Ok(enums::Chat::Chat(chat)) = result {
                    obj.set_telegram_chat(chat);
                }
            }),
        );
    }
}
