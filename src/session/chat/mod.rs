mod history;
mod item;
mod message;
mod sponsored_message;

use self::history::History;
pub use self::item::{Item, ItemType};
pub use self::message::{Message, MessageSender};
pub use self::sponsored_message::SponsoredMessage;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdgrand::enums::{self, ChatType, MessageContent, Update};
use tdgrand::types::{Chat as TelegramChat, ChatNotificationSettings, DraftMessage};

use crate::session::Avatar;
use crate::Session;

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "BoxedChatType")]
pub struct BoxedChatType(ChatType);

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedDraftMessage")]
pub struct BoxedDraftMessage(pub Option<DraftMessage>);

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "BoxedChatNotificationSettings")]
pub struct BoxedChatNotificationSettings(pub ChatNotificationSettings);

#[derive(Clone, Debug, PartialEq, glib::GBoxed)]
#[gboxed(type_name = "BoxedMessageContent")]
pub struct BoxedMessageContent(pub MessageContent);

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct Chat {
        pub id: Cell<i64>,
        pub type_: OnceCell<ChatType>,
        pub title: RefCell<String>,
        pub avatar: OnceCell<Avatar>,
        pub last_message: RefCell<Option<Message>>,
        pub order: Cell<i64>,
        pub is_pinned: Cell<bool>,
        pub unread_mention_count: Cell<i32>,
        pub unread_count: Cell<i32>,
        pub draft_message: RefCell<BoxedDraftMessage>,
        pub notification_settings: RefCell<Option<BoxedChatNotificationSettings>>,
        pub history: OnceCell<History>,
        pub session: WeakRef<Session>,
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
                    glib::ParamSpec::new_boxed(
                        "type",
                        "Type",
                        "The type of this chat",
                        BoxedChatType::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "The title of this chat",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpec::new_object(
                        "avatar",
                        "Avatar",
                        "The avatar of this chat",
                        Avatar::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "last-message",
                        "Last Message",
                        "The last message sent on this chat",
                        Message::static_type(),
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
                    glib::ParamSpec::new_boolean(
                        "is-pinned",
                        "Is Pinned",
                        "The parameter to determine if this chat is pinned in the chat list",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_int(
                        "unread-mention-count",
                        "Unread Mention Count",
                        "The unread mention count of this chat",
                        std::i32::MIN,
                        std::i32::MAX,
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
                    glib::ParamSpec::new_boxed(
                        "draft-message",
                        "Draft Message",
                        "The draft message of this chat",
                        BoxedDraftMessage::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boxed(
                        "notification-settings",
                        "Notification Settings",
                        "The notification settings of this chat",
                        BoxedChatNotificationSettings::static_type(),
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
                "id" => {
                    let id = value.get().unwrap();
                    self.id.set(id);
                }
                "type" => {
                    let type_ = value.get::<BoxedChatType>().unwrap();
                    self.type_.set(type_.0).unwrap();
                }
                "title" => {
                    let title = value.get().unwrap();
                    self.title.replace(title);
                }
                "avatar" => {
                    self.avatar.set(value.get().unwrap()).unwrap();
                }
                "last-message" => {
                    let last_message = value.get().unwrap();
                    self.last_message.replace(last_message);
                }
                "order" => {
                    let order = value.get().unwrap();
                    self.order.set(order);
                }
                "is-pinned" => {
                    let is_pinned = value.get().unwrap();
                    self.is_pinned.set(is_pinned);
                }
                "unread-mention-count" => {
                    let unread_mention_count = value.get().unwrap();
                    self.unread_mention_count.set(unread_mention_count);
                }
                "unread-count" => {
                    let unread_count = value.get().unwrap();
                    self.unread_count.set(unread_count);
                }
                "draft-message" => {
                    let draft_message = value.get().unwrap();
                    self.draft_message.replace(draft_message);
                }
                "notification-settings" => {
                    let notification_settings = value.get().unwrap();
                    self.notification_settings
                        .replace(Some(notification_settings));
                }
                "session" => self.session.set(Some(&value.get().unwrap())),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "title" => self.title.borrow().to_value(),
                "avatar" => obj.avatar().to_value(),
                "last-message" => self.last_message.borrow().to_value(),
                "order" => self.order.get().to_value(),
                "is-pinned" => self.is_pinned.get().to_value(),
                "unread-mention-count" => self.unread_mention_count.get().to_value(),
                "unread-count" => self.unread_count.get().to_value(),
                "draft-message" => self.draft_message.borrow().to_value(),
                "notification-settings" => self
                    .notification_settings
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .to_value(),
                "history" => self.history.get().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.history.set(History::new(obj)).unwrap();

            let avatar = obj.avatar();
            let title_expression = obj.title_expression();
            title_expression.bind(avatar, "display-name", Some(avatar));
        }
    }
}

glib::wrapper! {
    pub struct Chat(ObjectSubclass<imp::Chat>);
}

impl Chat {
    pub fn new(chat: TelegramChat, session: Session) -> Self {
        let type_ = BoxedChatType(chat.r#type);
        let avatar = Avatar::new(&session);
        avatar.update_from_chat_photo(chat.photo);

        glib::Object::new(&[
            ("id", &chat.id),
            ("type", &type_),
            ("title", &chat.title),
            ("avatar", &avatar),
            ("draft-message", &BoxedDraftMessage(chat.draft_message)),
            ("unread-mention-count", &chat.unread_mention_count),
            ("unread-count", &chat.unread_count),
            (
                "notification-settings",
                &BoxedChatNotificationSettings(chat.notification_settings),
            ),
            ("session", &session),
        ])
        .expect("Failed to create Chat")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::NewMessage(_)
            | Update::MessageSendSucceeded(_)
            | Update::MessageContent(_)
            | Update::DeleteMessages(_) => {
                self.history().handle_update(update);
            }
            Update::ChatTitle(update) => {
                self.set_title(update.title);
            }
            Update::ChatPhoto(update) => {
                self.avatar().update_from_chat_photo(update.photo);
            }
            Update::ChatLastMessage(update) => {
                match update.last_message {
                    Some(last_message) => {
                        let message = match self.history().message_by_id(last_message.id) {
                            Some(message) => message,
                            None => {
                                let last_message_id = last_message.id;

                                self.history().append(last_message);
                                self.history().message_by_id(last_message_id).unwrap()
                            }
                        };

                        self.set_last_message(Some(message));
                    }
                    None => self.set_last_message(None),
                }

                for position in update.positions {
                    if let enums::ChatList::Main = position.list {
                        self.set_order(position.order);
                        break;
                    }
                }
            }
            Update::ChatNotificationSettings(update) => {
                self.set_notification_settings(update.notification_settings);
            }
            Update::ChatPosition(update) => {
                if let enums::ChatList::Main = update.position.list {
                    self.set_order(update.position.order);
                    self.set_is_pinned(update.position.is_pinned);
                }
            }
            Update::ChatUnreadMentionCount(update) => {
                self.set_unread_mention_count(update.unread_mention_count);
            }
            Update::MessageMentionRead(update) => {
                self.set_unread_mention_count(update.unread_mention_count);
            }
            Update::ChatReadInbox(update) => {
                self.set_unread_count(update.unread_count);
            }
            Update::ChatDraftMessage(update) => {
                self.set_draft_message(BoxedDraftMessage(update.draft_message));
            }
            _ => {}
        }
    }

    pub fn id(&self) -> i64 {
        self.property("id").unwrap().get().unwrap()
    }

    pub fn type_(&self) -> &ChatType {
        let self_ = imp::Chat::from_instance(self);
        self_.type_.get().unwrap()
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }

    fn set_title(&self, title: String) {
        if self.title() != title {
            self.set_property("title", &title).unwrap();
        }
    }

    pub fn avatar(&self) -> &Avatar {
        let self_ = imp::Chat::from_instance(self);
        self_.avatar.get().unwrap()
    }

    pub fn last_message(&self) -> Option<Message> {
        self.property("last-message").unwrap().get().unwrap()
    }

    fn set_last_message(&self, last_message: Option<Message>) {
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

    pub fn is_pinned(&self) -> bool {
        self.property("is-pinned").unwrap().get().unwrap()
    }

    fn set_is_pinned(&self, is_pinned: bool) {
        if self.is_pinned() != is_pinned {
            self.set_property("is-pinned", &is_pinned).unwrap();
        }
    }

    pub fn unread_mention_count(&self) -> i32 {
        self.property("unread-mention-count")
            .unwrap()
            .get()
            .unwrap()
    }

    fn set_unread_mention_count(&self, unread_mention_count: i32) {
        if self.unread_mention_count() != unread_mention_count {
            self.set_property("unread-mention-count", &unread_mention_count)
                .unwrap();
        }
    }

    pub fn unread_count(&self) -> i32 {
        self.property("unread-count").unwrap().get().unwrap()
    }

    fn set_unread_count(&self, unread_count: i32) {
        if self.unread_count() != unread_count {
            self.set_property("unread-count", &unread_count).unwrap();
        }
    }

    pub fn draft_message(&self) -> BoxedDraftMessage {
        self.property("draft-message").unwrap().get().unwrap()
    }

    fn set_draft_message(&self, draft_message: BoxedDraftMessage) {
        if self.draft_message().0 != draft_message.0 {
            self.set_property("draft-message", &draft_message).unwrap();
        }
    }

    pub fn notification_settings(&self) -> ChatNotificationSettings {
        self.property("notification-settings")
            .unwrap()
            .get::<BoxedChatNotificationSettings>()
            .unwrap()
            .0
    }

    fn set_notification_settings(&self, notification_settings: ChatNotificationSettings) {
        if self.notification_settings() != notification_settings {
            self.set_property(
                "notification-settings",
                &BoxedChatNotificationSettings(notification_settings),
            )
            .unwrap();
        }
    }

    pub fn history(&self) -> History {
        self.property("history").unwrap().get().unwrap()
    }

    pub fn session(&self) -> Session {
        let self_ = imp::Chat::from_instance(self);
        self_.session.upgrade().unwrap()
    }

    pub fn title_expression(&self) -> gtk::Expression {
        let chat_expression = gtk::ConstantExpression::new(self);
        gtk::PropertyExpression::new(Chat::static_type(), Some(&chat_expression), "title").upcast()
    }
}
