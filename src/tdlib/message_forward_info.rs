use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::MessageForwardOrigin as TelegramMessageForwardOrigin;
use tdlib::types::MessageForwardInfo as TelegramForwardInfo;

use crate::tdlib::{Chat, User};

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
                    glib::ParamSpecInt::builder("date").read_only().build(),
                    glib::ParamSpecBoxed::builder::<MessageForwardOrigin>("origin")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

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
                MessageForwardOrigin::User(chat.session().user(data.sender_user_id))
            }
            TelegramMessageForwardOrigin::Chat(data) => MessageForwardOrigin::Chat {
                // author_signature: data.author_signature,
                chat: chat.session().chat(data.sender_chat_id),
            },
            TelegramMessageForwardOrigin::Channel(data) => {
                let chat = chat.session().chat(data.chat_id);
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

        let message_forward_info: MessageForwardInfo = glib::Object::new();
        let imp = message_forward_info.imp();

        imp.date.set(forward_info.date);
        imp.origin.set(origin).unwrap();

        message_forward_info
    }

    pub(crate) fn date(&self) -> i32 {
        self.imp().date.get()
    }

    pub(crate) fn origin(&self) -> &MessageForwardOrigin {
        self.imp().origin.get().unwrap()
    }
}
