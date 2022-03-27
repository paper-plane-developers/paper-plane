use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::MessageForwardOrigin as TelegramMessageForwardOrigin;
use tdlib::types::MessageForwardInfo as TelegramForwardInfo;

use crate::session::chat::Chat;
use crate::session::User;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "MessageForwardOrigin")]
pub(crate) enum MessageForwardOrigin {
    User(User),
    Chat {
        // author_signature: String,
        chat: Chat,
    },
    Channel {
        // author_signature: String,
        chat: Chat,
        // Using a WeakRef here as messages can be deleted.
        // message: WeakRef<Message>,
    },
    HiddenUser {
        sender_name: String,
    },
    MessageImport {
        sender_name: String,
    },
}

impl MessageForwardOrigin {
    pub(crate) fn id(&self) -> Option<i64> {
        Some(match self {
            Self::User(user) => user.id(),
            Self::Chat { chat, .. } | Self::Channel { chat, .. } => chat.id(),
            _ => return None,
        })
    }
}

mod imp {
    use super::*;

    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub(crate) struct MessageForwardInfo {
        pub(super) date: Cell<i32>,
        pub(super) origin: OnceCell<MessageForwardOrigin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageForwardInfo {
        const NAME: &'static str = "MessageForwardInfo";
        type Type = super::MessageForwardInfo;
    }

    impl ObjectImpl for MessageForwardInfo {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt::new(
                        "date",
                        "Date",
                        "The date when the message was originally sent",
                        0,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "origin",
                        "Origin",
                        "The origin of the forwarded message",
                        MessageForwardOrigin::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "date" => self.date.set(value.get().unwrap()),
                "origin" => self.origin.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "date" => obj.date().to_value(),
                "origin" => obj.origin().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageForwardInfo(ObjectSubclass<imp::MessageForwardInfo>);
}

impl MessageForwardInfo {
    pub(crate) fn from_td_object(forward_info: TelegramForwardInfo, chat: &Chat) -> Self {
        let origin = match forward_info.origin {
            TelegramMessageForwardOrigin::User(data) => {
                MessageForwardOrigin::User(chat.session().user_list().get(data.sender_user_id))
            }
            TelegramMessageForwardOrigin::Chat(data) => MessageForwardOrigin::Chat {
                // author_signature: data.author_signature,
                chat: chat.session().chat_list().get(data.sender_chat_id),
            },
            TelegramMessageForwardOrigin::Channel(data) => {
                let chat = chat.session().chat_list().get(data.chat_id);
                // let message = {
                //     let weak = WeakRef::new();
                //     weak.set(chat.history().message_by_id(data.message_id).as_ref());
                //     weak
                // };
                MessageForwardOrigin::Channel {
                    // author_signature: data.author_signature,
                    chat,
                    // message,
                }
            }
            TelegramMessageForwardOrigin::HiddenUser(data) => MessageForwardOrigin::HiddenUser {
                sender_name: data.sender_name,
            },
            TelegramMessageForwardOrigin::MessageImport(data) => {
                MessageForwardOrigin::MessageImport {
                    sender_name: data.sender_name,
                }
            }
        };

        glib::Object::new(&[("date", &forward_info.date), ("origin", &origin)])
            .expect("Failed to create MessageForwardInfo")
    }

    pub(crate) fn date(&self) -> i32 {
        self.imp().date.get()
    }

    pub(crate) fn origin(&self) -> &MessageForwardOrigin {
        self.imp().origin.get().unwrap()
    }
}
