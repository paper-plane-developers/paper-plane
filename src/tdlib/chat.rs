use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::{ChatType as TdChatType, Update};
use tdlib::types::Chat as TelegramChat;
use tdlib::{functions, types};

use crate::tdlib::{
    Avatar, BasicGroup, BoxedChatNotificationSettings, BoxedChatPermissions, BoxedDraftMessage,
    ChatActionList, ChatHistory, Message, SecretChat, Supergroup, User,
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
                let user = session.user(data.user_id);
                Self::Private(user)
            }
            TdChatType::BasicGroup(data) => {
                let basic_group = session.basic_group(data.basic_group_id);
                Self::BasicGroup(basic_group)
            }
            TdChatType::Supergroup(data) => {
                let supergroup = session.supergroup(data.supergroup_id);
                Self::Supergroup(supergroup)
            }
            TdChatType::Secret(data) => {
                let secret_chat = session.secret_chat(data.secret_chat_id);
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
        pub(super) last_read_outbox_message_id: Cell<i64>,
        pub(super) is_marked_as_unread: Cell<bool>,
        pub(super) last_message: RefCell<Option<Message>>,
        pub(super) unread_mention_count: Cell<i32>,
        pub(super) unread_count: Cell<i32>,
        pub(super) draft_message: RefCell<Option<BoxedDraftMessage>>,
        pub(super) notification_settings: RefCell<Option<BoxedChatNotificationSettings>>,
        pub(super) history: OnceCell<ChatHistory>,
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
                    glib::ParamSpecInt64::builder("id").read_only().build(),
                    glib::ParamSpecBoxed::builder::<ChatType>("type")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-blocked")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("title").read_only().build(),
                    glib::ParamSpecBoxed::builder::<Avatar>("avatar")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt64::builder("last-read-outbox-message-id")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-marked-as-unread")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<Message>("last-message")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt::builder("unread-mention-count")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt::builder("unread-count")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<BoxedDraftMessage>("draft-message")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<BoxedChatNotificationSettings>(
                        "notification-settings",
                    )
                    .read_only()
                    .build(),
                    glib::ParamSpecObject::builder::<ChatHistory>("history")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<ChatActionList>("actions")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<BoxedChatPermissions>("permissions")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<Session>("session")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "id" => obj.id().to_value(),
                "type" => obj.type_().to_value(),
                "is-blocked" => obj.is_blocked().to_value(),
                "title" => obj.title().to_value(),
                "avatar" => obj.avatar().to_value(),
                "last-read-outbox-message-id" => obj.last_read_outbox_message_id().to_value(),
                "is-marked-as-unread" => obj.is_marked_as_unread().to_value(),
                "last-message" => obj.last_message().to_value(),
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
    pub(crate) fn new(td_chat: TelegramChat, session: &Session) -> Self {
        let chat: Chat = glib::Object::builder().build();
        let imp = chat.imp();

        let type_ = ChatType::from_td_object(&td_chat.r#type, session);
        let avatar = td_chat.photo.map(Avatar::from);
        let last_message = td_chat.last_message.map(|m| Message::new(m, &chat));
        let draft_message = td_chat.draft_message.map(BoxedDraftMessage);
        let notification_settings = BoxedChatNotificationSettings(td_chat.notification_settings);
        let permissions = BoxedChatPermissions(td_chat.permissions);

        imp.id.set(td_chat.id);
        imp.type_.set(type_).unwrap();
        imp.is_blocked.set(td_chat.is_blocked);
        imp.title.replace(td_chat.title);
        imp.avatar.replace(avatar);
        imp.last_read_outbox_message_id
            .set(td_chat.last_read_outbox_message_id);
        imp.is_marked_as_unread.set(td_chat.is_marked_as_unread);
        imp.last_message.replace(last_message);
        imp.unread_mention_count.set(td_chat.unread_mention_count);
        imp.unread_count.set(td_chat.unread_count);
        imp.draft_message.replace(draft_message);
        imp.notification_settings
            .replace(Some(notification_settings));
        imp.session.set(Some(session));
        imp.permissions.replace(Some(permissions));

        chat
    }

    pub(crate) fn handle_update(&self, update: Update) {
        use Update::*;

        match update {
            ChatAction(update) => {
                self.actions().handle_update(update);
                // TODO: Remove this at some point. Widgets should use the `items-changed` signal
                // for updating their state in the future.
                self.notify("actions");
            }
            ChatDraftMessage(update) => {
                self.set_draft_message(update.draft_message.map(BoxedDraftMessage));
            }
            ChatIsBlocked(update) => self.set_is_blocked(update.is_blocked),
            ChatIsMarkedAsUnread(update) => self.set_marked_as_unread(update.is_marked_as_unread),
            ChatLastMessage(update) => {
                self.set_last_message(update.last_message.map(|m| Message::new(m, self)));
            }
            ChatNotificationSettings(update) => {
                self.set_notification_settings(BoxedChatNotificationSettings(
                    update.notification_settings,
                ));
            }
            ChatPermissions(update) => {
                self.set_permissions(BoxedChatPermissions(update.permissions))
            }
            ChatPhoto(update) => self.set_avatar(update.photo.map(Into::into)),
            ChatReadInbox(update) => self.set_unread_count(update.unread_count),
            ChatReadOutbox(update) => {
                self.set_last_read_outbox_message_id(update.last_read_outbox_message_id);
            }
            ChatTitle(update) => self.set_title(update.title),
            ChatUnreadMentionCount(update) => {
                self.set_unread_mention_count(update.unread_mention_count)
            }
            DeleteMessages(_)
            | MessageContent(_)
            | MessageEdited(_)
            | MessageSendSucceeded(_)
            | NewMessage(_) => {
                self.history().handle_update(update);
            }
            MessageMentionRead(update) => {
                self.set_unread_mention_count(update.unread_mention_count)
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

    fn set_is_blocked(&self, is_blocked: bool) {
        if self.is_blocked() == is_blocked {
            return;
        }
        self.imp().is_blocked.replace(is_blocked);
        self.notify("is-blocked");
    }

    pub(crate) fn title(&self) -> String {
        self.imp().title.borrow().to_owned()
    }

    fn set_title(&self, title: String) {
        if self.title() == title {
            return;
        }
        self.imp().title.replace(title);
        self.notify("title");
    }

    pub(crate) fn avatar(&self) -> Option<Avatar> {
        self.imp().avatar.borrow().to_owned()
    }

    fn set_avatar(&self, avatar: Option<Avatar>) {
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

    pub(crate) fn last_read_outbox_message_id(&self) -> i64 {
        self.imp().last_read_outbox_message_id.get()
    }

    fn set_last_read_outbox_message_id(&self, last_read_outbox_message_id: i64) {
        if self.last_read_outbox_message_id() == last_read_outbox_message_id {
            return;
        }
        self.imp()
            .last_read_outbox_message_id
            .set(last_read_outbox_message_id);
        self.notify("last-read-outbox-message-id");
    }

    pub(crate) fn is_marked_as_unread(&self) -> bool {
        self.imp().is_marked_as_unread.get()
    }

    fn set_marked_as_unread(&self, is_marked_as_unread: bool) {
        if self.is_marked_as_unread() == is_marked_as_unread {
            return;
        }
        self.imp().is_marked_as_unread.set(is_marked_as_unread);
        self.notify("is-marked-as-unread");
    }

    pub(crate) fn last_message(&self) -> Option<Message> {
        self.imp().last_message.borrow().to_owned()
    }

    fn set_last_message(&self, last_message: Option<Message>) {
        if self.last_message() == last_message {
            return;
        }
        self.imp().last_message.replace(last_message);
        self.notify("last-message");
    }

    pub(crate) fn unread_mention_count(&self) -> i32 {
        self.imp().unread_mention_count.get()
    }

    fn set_unread_mention_count(&self, unread_mention_count: i32) {
        if self.unread_mention_count() == unread_mention_count {
            return;
        }
        self.imp().unread_mention_count.set(unread_mention_count);
        self.notify("unread-mention-count");
    }

    pub(crate) fn unread_count(&self) -> i32 {
        self.imp().unread_count.get()
    }

    fn set_unread_count(&self, unread_count: i32) {
        if self.unread_count() == unread_count {
            return;
        }
        self.imp().unread_count.set(unread_count);
        self.notify("unread-count");
    }

    pub(crate) fn draft_message(&self) -> Option<BoxedDraftMessage> {
        self.imp().draft_message.borrow().to_owned()
    }

    fn set_draft_message(&self, draft_message: Option<BoxedDraftMessage>) {
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

    fn set_notification_settings(&self, notification_settings: BoxedChatNotificationSettings) {
        if self.imp().notification_settings.borrow().as_ref() == Some(&notification_settings) {
            return;
        }
        self.imp()
            .notification_settings
            .replace(Some(notification_settings));
        self.notify("notification-settings");
    }

    pub(crate) fn history(&self) -> &ChatHistory {
        self.imp().history.get_or_init(|| ChatHistory::new(self))
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

    fn set_permissions(&self, permissions: BoxedChatPermissions) {
        if self.imp().permissions.borrow().as_ref() == Some(&permissions) {
            return;
        }
        self.imp().permissions.replace(Some(permissions));
        self.notify("permissions");
    }

    pub(crate) async fn mark_as_read(&self) -> Result<(), types::Error> {
        if let Some(message) = self.last_message() {
            functions::view_messages(
                self.id(),
                0,
                vec![message.id()],
                true,
                self.session().client_id(),
            )
            .await?;
        }

        functions::toggle_chat_is_marked_as_unread(self.id(), false, self.session().client_id())
            .await
    }

    pub(crate) async fn mark_as_unread(&self) -> Result<(), types::Error> {
        functions::toggle_chat_is_marked_as_unread(self.id(), true, self.session().client_id())
            .await
    }
}
