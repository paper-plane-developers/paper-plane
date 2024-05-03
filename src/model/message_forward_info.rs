use std::cell::OnceCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;
use crate::types::MessageId;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "MessageForwardOrigin")]
pub(crate) enum MessageForwardOrigin {
    User(model::User),
    Chat {
        // author_signature: String,
        chat: model::Chat,
    },
    Channel {
        // author_signature: String,
        chat: model::Chat,
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
    pub(crate) fn id(&self) -> Option<MessageId> {
        Some(match self {
            Self::User(user) => user.id(),
            Self::Chat { chat, .. } | Self::Channel { chat, .. } => chat.id(),
            _ => return None,
        })
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::MessageForwardInfo)]
    pub(crate) struct MessageForwardInfo {
        #[property(get, set, construct_only)]
        pub(super) date: OnceCell<i32>,
        #[property(get, set, construct_only)]
        pub(super) origin: OnceCell<model::MessageForwardOrigin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageForwardInfo {
        const NAME: &'static str = "MessageForwardInfo";
        type Type = super::MessageForwardInfo;
    }

    impl ObjectImpl for MessageForwardInfo {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageForwardInfo(ObjectSubclass<imp::MessageForwardInfo>);
}

impl MessageForwardInfo {
    pub(crate) fn new(chat: &model::Chat, forward_info: tdlib::types::MessageForwardInfo) -> Self {
        use tdlib::enums::MessageForwardOrigin::*;

        let origin = match forward_info.origin {
            User(data) => MessageForwardOrigin::User(chat.session_().user(data.sender_user_id)),
            Chat(data) => MessageForwardOrigin::Chat {
                // author_signature: data.author_signature,
                chat: chat.session_().chat(data.sender_chat_id),
            },
            Channel(data) => {
                let chat = chat.session_().chat(data.chat_id);
                // let message = {
                //     let weak = glib::WeakRef::new();
                //     weak.set(chat.history().message_by_id(data.message_id).as_ref());
                //     weak
                // };
                MessageForwardOrigin::Channel {
                    // author_signature: data.author_signature,
                    chat,
                    // message,
                }
            }
            HiddenUser(data) => MessageForwardOrigin::HiddenUser {
                sender_name: data.sender_name,
            },
            MessageImport(data) => MessageForwardOrigin::MessageImport {
                sender_name: data.sender_name,
            },
        };

        glib::Object::builder()
            .property("date", forward_info.date)
            .property("origin", origin)
            .build()
    }
}
