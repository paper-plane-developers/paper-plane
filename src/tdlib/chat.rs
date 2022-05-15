use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::{self, ChatType as TdChatType, Update};
use tdlib::types::Chat as TelegramChat;

use crate::tdlib::{
    Avatar, BasicGroup, BoxedChatNotificationSettings, BoxedChatPermissions, BoxedDraftMessage,
    ChatActionList, History, Message, SecretChat, Supergroup, User,
};
use crate::Session;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "ChatType")]
pub(crate) enum ChatType {
    Private(User),
    BasicGroup(BasicGroup),
    Supergroup(Supergroup),
    Secret(SecretChat),
}

impl ChatType {
    pub(crate) fn from_td_object(_type: &TdChatType, session: &Session) -> Self {
        match _type {
            TdChatType::Private(data) => {
                let user = session.user_list().get(data.user_id);
                Self::Private(user)
            }
            TdChatType::BasicGroup(data) => {
                let basic_group = session.basic_group_list().get(data.basic_group_id);
                Self::BasicGroup(basic_group)
            }
            TdChatType::Supergroup(data) => {
                let supergroup = session.supergroup_list().get(data.supergroup_id);
                Self::Supergroup(supergroup)
            }
            TdChatType::Secret(data) => {
                let secret_chat = session.secret_chat_list().get(data.secret_chat_id);
                Self::Secret(secret_chat)
            }
        }
    }

    pub(crate) fn user(&self) -> Option<&User> {
        Some(match self {
            ChatType::Private(user) => user,
            ChatType::Secret(secret_chat) => secret_chat.user(),
            _ => return None,
        })
    }
}

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct Chat {
        pub(super) id: Cell<i64>,
        pub(super) type_: OnceCell<ChatType>,
        pub(super) is_blocked: Cell<bool>,
        pub(super) title: RefCell<String>,
        pub(super) avatar: RefCell<Option<Avatar>>,
        pub(super) last_message: RefCell<Option<Message>>,
        pub(super) order: Cell<i64>,
        pub(super) is_pinned: Cell<bool>,
        pub(super) unread_mention_count: Cell<i32>,
        pub(super) unread_count: Cell<i32>,
        pub(super) draft_message: RefCell<Option<BoxedDraftMessage>>,
        pub(super) notification_settings: RefCell<Option<BoxedChatNotificationSettings>>,
        pub(super) history: OnceCell<History>,
        pub(super) actions: OnceCell<ChatActionList>,
        pub(super) session: WeakRef<Session>,
        pub(super) permissions: RefCell<Option<BoxedChatPermissions>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Chat {
        const NAME: &'static str = "Chat";
        type Type = super::Chat;
    }

    impl ObjectImpl for Chat {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::new(
                        "id",
                        "Id",
                        "The id of this chat",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "type",
                        "Type",
                        "The type of this chat",
                        ChatType::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-blocked",
                        "Is blocked",
                        "Whether this chat is blocked for the user",
                        false,
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "title",
                        "Title",
                        "The title of this chat",
                        Some(""),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "avatar",
                        "Avatar",
                        "The avatar of this chat",
                        Avatar::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "last-message",
                        "Last Message",
                        "The last message sent on this chat",
                        Message::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecInt64::new(
                        "order",
                        "Order",
                        "The parameter to determine the order of this chat in the chat list",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-pinned",
                        "Is Pinned",
                        "The parameter to determine if this chat is pinned in the chat list",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecInt::new(
                        "unread-mention-count",
                        "Unread Mention Count",
                        "The unread mention count of this chat",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecInt::new(
                        "unread-count",
                        "Unread Count",
                        "The unread messages count of this chat",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "draft-message",
                        "Draft Message",
                        "The draft message of this chat",
                        BoxedDraftMessage::static_type(),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "notification-settings",
                        "Notification Settings",
                        "The notification settings of this chat",
                        BoxedChatNotificationSettings::static_type(),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "history",
                        "History",
                        "The message history of this chat",
                        History::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        "actions",
                        "Actions",
                        "The chronologically ordered actions of this chat",
                        ChatActionList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "permissions",
                        "Permissions",
                        "The permissions of this chat",
                        BoxedChatPermissions::static_type(),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "id" => self.id.set(value.get().unwrap()),
                "type" => self.type_.set(value.get().unwrap()).unwrap(),
                "is-blocked" => obj.set_is_blocked(value.get().unwrap()),
                "title" => {
                    obj.set_title(value.get::<Option<String>>().unwrap().unwrap_or_default())
                }
                "avatar" => obj.set_avatar(value.get().unwrap()),
                "last-message" => obj.set_last_message(value.get().unwrap()),
                "order" => obj.set_order(value.get().unwrap()),
                "is-pinned" => obj.set_is_pinned(value.get().unwrap()),
                "unread-mention-count" => obj.set_unread_mention_count(value.get().unwrap()),
                "unread-count" => obj.set_unread_count(value.get().unwrap()),
                "draft-message" => obj.set_draft_message(value.get().unwrap()),
                "notification-settings" => obj.set_notification_settings(value.get().unwrap()),
                "permissions" => obj.set_permissions(value.get().unwrap()),
                "session" => self.session.set(Some(&value.get().unwrap())),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "type" => obj.type_().to_value(),
                "is-blocked" => obj.is_blocked().to_value(),
                "title" => obj.title().to_value(),
                "avatar" => obj.avatar().to_value(),
                "last-message" => obj.last_message().to_value(),
                "order" => obj.order().to_value(),
                "is-pinned" => obj.is_pinned().to_value(),
                "unread-mention-count" => obj.unread_mention_count().to_value(),
                "unread-count" => obj.unread_count().to_value(),
                "draft-message" => obj.draft_message().to_value(),
                "notification-settings" => obj.notification_settings().to_value(),
                "history" => obj.history().to_value(),
                "actions" => obj.actions().to_value(),
                "permissions" => obj.permissions().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Chat(ObjectSubclass<imp::Chat>);
}

impl Chat {
    pub(crate) fn new(chat: TelegramChat, session: Session) -> Self {
        let type_ = ChatType::from_td_object(&chat.r#type, &session);
        let avatar = chat.photo.map(Avatar::from);
        let draft_message = chat.draft_message.map(BoxedDraftMessage);

        glib::Object::new(&[
            ("id", &chat.id),
            ("type", &type_),
            ("is-blocked", &chat.is_blocked),
            ("title", &chat.title),
            ("avatar", &avatar),
            ("draft-message", &draft_message),
            ("unread-mention-count", &chat.unread_mention_count),
            ("unread-count", &chat.unread_count),
            (
                "notification-settings",
                &BoxedChatNotificationSettings(chat.notification_settings),
            ),
            ("permissions", &BoxedChatPermissions(chat.permissions)),
            ("session", &session),
        ])
        .expect("Failed to create Chat")
    }

    pub(crate) fn handle_update(&self, update: Update) {
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
                self.set_avatar(update.photo.map(Into::into));
            }
            Update::ChatLastMessage(update) => {
                match update.last_message {
                    Some(last_message) => {
                        let message = match self.history().message_by_id(last_message.id) {
                            Some(message) => message,
                            None => {
                                let last_message_id = last_message.id;

                                self.history().push_front(last_message);
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
                self.set_notification_settings(BoxedChatNotificationSettings(
                    update.notification_settings,
                ));
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
                self.set_draft_message(update.draft_message.map(BoxedDraftMessage));
            }
            Update::ChatAction(update) => {
                self.actions().handle_update(update);
                // TODO: Remove this at some point. Widgets should use the `items-changed` signal
                // for updating their state in the future.
                self.notify("actions");
            }
            Update::ChatIsBlocked(update) => self.set_is_blocked(update.is_blocked),
            Update::ChatPermissions(update) => {
                self.set_permissions(BoxedChatPermissions(update.permissions))
            }
            _ => {}
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.imp().id.get()
    }

    pub(crate) fn type_(&self) -> &ChatType {
        self.imp().type_.get().unwrap()
    }

    pub(crate) fn is_blocked(&self) -> bool {
        self.imp().is_blocked.get()
    }

    pub(crate) fn set_is_blocked(&self, is_blocked: bool) {
        if self.is_blocked() == is_blocked {
            return;
        }
        self.imp().is_blocked.replace(is_blocked);
        self.notify("is-blocked");
    }

    pub(crate) fn title(&self) -> String {
        self.imp().title.borrow().to_owned()
    }

    pub(crate) fn set_title(&self, title: String) {
        if self.title() == title {
            return;
        }
        self.imp().title.replace(title);
        self.notify("title");
    }

    pub(crate) fn avatar(&self) -> Option<Avatar> {
        self.imp().avatar.borrow().to_owned()
    }

    pub(crate) fn set_avatar(&self, avatar: Option<Avatar>) {
        if self.avatar() == avatar {
            return;
        }
        self.imp().avatar.replace(avatar);
        self.notify("avatar");
    }

    pub(crate) fn connect_avatar_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("avatar"), f)
    }

    pub(crate) fn last_message(&self) -> Option<Message> {
        self.imp().last_message.borrow().to_owned()
    }

    pub(crate) fn set_last_message(&self, last_message: Option<Message>) {
        if self.last_message() == last_message {
            return;
        }
        self.imp().last_message.replace(last_message);
        self.notify("last-message");
    }

    pub(crate) fn order(&self) -> i64 {
        self.imp().order.get()
    }

    pub(crate) fn set_order(&self, order: i64) {
        if self.order() == order {
            return;
        }
        self.imp().order.set(order);
        self.notify("order");
    }

    pub(crate) fn connect_order_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("order"), f)
    }

    pub(crate) fn is_pinned(&self) -> bool {
        self.imp().is_pinned.get()
    }

    pub(crate) fn set_is_pinned(&self, is_pinned: bool) {
        if self.is_pinned() == is_pinned {
            return;
        }
        self.imp().is_pinned.set(is_pinned);
        self.notify("is-pinned");
    }

    pub(crate) fn unread_mention_count(&self) -> i32 {
        self.imp().unread_mention_count.get()
    }

    pub(crate) fn set_unread_mention_count(&self, unread_mention_count: i32) {
        if self.unread_mention_count() == unread_mention_count {
            return;
        }
        self.imp().unread_mention_count.set(unread_mention_count);
        self.notify("unread-mention-count");
    }

    pub(crate) fn unread_count(&self) -> i32 {
        self.imp().unread_count.get()
    }

    pub(crate) fn set_unread_count(&self, unread_count: i32) {
        if self.unread_count() == unread_count {
            return;
        }
        self.imp().unread_count.set(unread_count);
        self.notify("unread-count");
    }

    pub(crate) fn draft_message(&self) -> Option<BoxedDraftMessage> {
        self.imp().draft_message.borrow().to_owned()
    }

    pub(crate) fn set_draft_message(&self, draft_message: Option<BoxedDraftMessage>) {
        if self.draft_message() == draft_message {
            return;
        }
        self.imp().draft_message.replace(draft_message);
        self.notify("draft-message");
    }

    pub(crate) fn notification_settings(&self) -> BoxedChatNotificationSettings {
        self.imp()
            .notification_settings
            .borrow()
            .as_ref()
            .unwrap()
            .to_owned()
    }

    pub(crate) fn set_notification_settings(
        &self,
        notification_settings: BoxedChatNotificationSettings,
    ) {
        if self.imp().notification_settings.borrow().as_ref() == Some(&notification_settings) {
            return;
        }
        self.imp()
            .notification_settings
            .replace(Some(notification_settings));
        self.notify("notification-settings");
    }

    pub(crate) fn history(&self) -> &History {
        self.imp().history.get_or_init(|| History::new(self))
    }

    pub(crate) fn actions(&self) -> &ChatActionList {
        self.imp()
            .actions
            .get_or_init(|| ChatActionList::from(self))
    }

    pub(crate) fn session(&self) -> Session {
        self.imp().session.upgrade().unwrap()
    }

    pub(crate) fn is_own_chat(&self) -> bool {
        self.type_().user() == Some(&self.session().me())
    }

    pub(crate) fn permissions(&self) -> BoxedChatPermissions {
        self.imp().permissions.borrow().to_owned().unwrap()
    }

    pub(crate) fn set_permissions(&self, permissions: BoxedChatPermissions) {
        if self.imp().permissions.borrow().as_ref() == Some(&permissions) {
            return;
        }
        self.imp().permissions.replace(Some(permissions));
        self.notify("permissions");
    }
}
