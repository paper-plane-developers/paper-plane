mod components;
mod content;
mod sidebar;

use self::content::Content;
use self::sidebar::Sidebar;

use glib::{clone, SyncSender};
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use std::collections::hash_map::{Entry, HashMap};
use tdlib::enums::{self, NotificationSettingsScope, Update};
use tdlib::functions;
use tdlib::types::File;

use crate::session_manager::DatabaseInfo;
use crate::tdlib::{
    BasicGroup, BoxedScopeNotificationSettings, ChatList, SecretChat, Supergroup, User, UserList,
};
use crate::utils::{log_out, spawn};

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedDatabaseInfo")]
pub(crate) struct BoxedDatabaseInfo(pub(crate) DatabaseInfo);

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/session.ui")]
    pub(crate) struct Session {
        pub(super) client_id: Cell<i32>,
        pub(super) database_info: OnceCell<BoxedDatabaseInfo>,
        pub(super) me: WeakRef<User>,
        pub(super) chat_list: OnceCell<ChatList>,
        pub(super) user_list: OnceCell<UserList>,
        pub(super) basic_groups: RefCell<HashMap<i64, BasicGroup>>,
        pub(super) supergroups: RefCell<HashMap<i64, Supergroup>>,
        pub(super) secret_chats: RefCell<HashMap<i32, SecretChat>>,
        pub(super) private_chats_notification_settings:
            RefCell<Option<BoxedScopeNotificationSettings>>,
        pub(super) group_chats_notification_settings:
            RefCell<Option<BoxedScopeNotificationSettings>>,
        pub(super) channel_chats_notification_settings:
            RefCell<Option<BoxedScopeNotificationSettings>>,
        pub(super) downloading_files: RefCell<HashMap<i32, Vec<SyncSender<File>>>>,
        #[template_child]
        pub(super) leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub(super) sidebar: TemplateChild<Sidebar>,
        #[template_child]
        pub(super) content: TemplateChild<Content>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Session {
        const NAME: &'static str = "Session";
        type Type = super::Session;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("content.go-back", None, move |widget, _, _| {
                widget
                    .imp()
                    .leaflet
                    .navigate(adw::NavigationDirection::Back);
            });
            klass.install_action("session.log-out", None, move |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    log_out(widget.client_id()).await;
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Session {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt::new(
                        "client-id",
                        "Client Id",
                        "The client id",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "database-info",
                        "Database Info",
                        "The information about the database of this session",
                        BoxedDatabaseInfo::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "me",
                        "Me",
                        "The own user id of this session",
                        User::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        "chat-list",
                        "Chat List",
                        "A list of chats",
                        ChatList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        "user-list",
                        "User List",
                        "The list of users of this session",
                        UserList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        "private-chats-notification-settings",
                        "Private Chats Notification Settings",
                        "This session's notification settings for private chats",
                        BoxedScopeNotificationSettings::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "group-chats-notification-settings",
                        "Group Chats Notification Settings",
                        "This session's notification settings for group chats",
                        BoxedScopeNotificationSettings::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "channel-chats-notification-settings",
                        "Channel Chats Notification Settings",
                        "This session's notification settings for channel chats",
                        BoxedScopeNotificationSettings::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "client-id" => {
                    let client_id = value.get().unwrap();
                    self.client_id.set(client_id);
                }
                "database-info" => {
                    let database_info = value.get().unwrap();
                    self.database_info.set(database_info).unwrap();
                }
                "private-chats-notification-settings" => {
                    let scope_notification_settings = value.get().unwrap();
                    obj.set_private_chats_notification_settings(scope_notification_settings);
                }
                "group-chats-notification-settings" => {
                    let scope_notification_settings = value.get().unwrap();
                    obj.set_group_chats_notification_settings(scope_notification_settings);
                }
                "channel-chats-notification-settings" => {
                    let scope_notification_settings = value.get().unwrap();
                    obj.set_channel_chats_notification_settings(scope_notification_settings);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "client-id" => obj.client_id().to_value(),
                "database-info" => obj.database_info().to_value(),
                "me" => self.me.upgrade().to_value(),
                "chat-list" => obj.chat_list().to_value(),
                "user-list" => obj.user_list().to_value(),
                "private-chats-notification-settings" => {
                    obj.private_chats_notification_settings().to_value()
                }
                "group-chats-notification-settings" => {
                    obj.group_chats_notification_settings().to_value()
                }
                "channel-chats-notification-settings" => {
                    obj.channel_chats_notification_settings().to_value()
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.sidebar
                .connect_list_activated(clone!(@weak obj => move |_| {
                    obj.imp().leaflet.navigate(adw::NavigationDirection::Forward);
                }));
        }
    }

    impl WidgetImpl for Session {}
    impl BinImpl for Session {}
}

glib::wrapper! {
    pub(crate) struct Session(ObjectSubclass<imp::Session>)
        @extends gtk::Widget, adw::Bin;
}

impl Session {
    pub(crate) fn new(client_id: i32, database_info: DatabaseInfo) -> Self {
        glib::Object::new(&[
            ("client-id", &client_id),
            ("database-info", &BoxedDatabaseInfo(database_info)),
        ])
        .expect("Failed to create Session")
    }

    pub(crate) fn handle_update(&self, update: Update) {
        match update {
            Update::BasicGroup(data) => {
                let mut basic_groups = self.imp().basic_groups.borrow_mut();
                match basic_groups.entry(data.basic_group.id) {
                    Entry::Occupied(entry) => entry.get().update(data.basic_group),
                    Entry::Vacant(entry) => {
                        entry.insert(BasicGroup::from_td_object(data.basic_group));
                    }
                }
            }
            Update::ChatAction(_)
            | Update::ChatDraftMessage(_)
            | Update::ChatIsBlocked(_)
            | Update::ChatLastMessage(_)
            | Update::ChatNotificationSettings(_)
            | Update::ChatPermissions(_)
            | Update::ChatPhoto(_)
            | Update::ChatPosition(_)
            | Update::ChatReadInbox(_)
            | Update::ChatReadOutbox(_)
            | Update::ChatTitle(_)
            | Update::ChatUnreadMentionCount(_)
            | Update::DeleteMessages(_)
            | Update::MessageContent(_)
            | Update::MessageEdited(_)
            | Update::MessageMentionRead(_)
            | Update::MessageSendSucceeded(_)
            | Update::NewChat(_)
            | Update::NewMessage(_) => {
                self.chat_list().handle_update(update);
            }
            Update::File(update) => {
                self.handle_file_update(update.file);
            }
            Update::ScopeNotificationSettings(update) => {
                let settings = Some(BoxedScopeNotificationSettings(update.notification_settings));
                match update.scope {
                    NotificationSettingsScope::PrivateChats => {
                        self.set_private_chats_notification_settings(settings);
                    }
                    NotificationSettingsScope::GroupChats => {
                        self.set_group_chats_notification_settings(settings);
                    }
                    NotificationSettingsScope::ChannelChats => {
                        self.set_channel_chats_notification_settings(settings);
                    }
                }
            }
            Update::SecretChat(ref data) => {
                let mut secret_chats = self.imp().secret_chats.borrow_mut();
                match secret_chats.entry(data.secret_chat.id) {
                    Entry::Occupied(entry) => entry.get().handle_update(&update),
                    Entry::Vacant(entry) => {
                        let user = self.user_list().get(data.secret_chat.user_id);
                        entry.insert(SecretChat::from_td_object(&data.secret_chat, &user));
                    }
                }
            }
            Update::Supergroup(data) => {
                let mut supergroups = self.imp().supergroups.borrow_mut();
                match supergroups.entry(data.supergroup.id) {
                    Entry::Occupied(entry) => entry.get().update(data.supergroup),
                    Entry::Vacant(entry) => {
                        entry.insert(Supergroup::from_td_object(data.supergroup));
                    }
                }
            }
            Update::UnreadMessageCount(ref update_) => {
                // TODO: Also handle archived chats
                if let tdlib::enums::ChatList::Main = update_.chat_list {
                    self.chat_list().handle_update(update)
                }
            }
            Update::User(_) | Update::UserStatus(_) => {
                self.user_list().handle_update(update);
            }
            _ => {}
        }
    }

    /// Returns the `BasicGroup` of the specified id. Panics if the basic group is not present.
    ///
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an id returned by TDLib, it should be expected that the
    /// relative `BasicGroup` exists in the list.
    pub(crate) fn basic_group(&self, basic_group_id: i64) -> BasicGroup {
        self.imp()
            .basic_groups
            .borrow()
            .get(&basic_group_id)
            .expect("Failed to get expected BasicGroup")
            .clone()
    }

    /// Returns the `Supergroup` of the specified id. Panics if the supergroup is not present.
    ///
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an id returned by TDLib, it should be expected that the
    /// relative `Supergroup` exists in the list.
    pub(crate) fn supergroup(&self, supergroup_id: i64) -> Supergroup {
        self.imp()
            .supergroups
            .borrow()
            .get(&supergroup_id)
            .expect("Failed to get expected Supergroup")
            .clone()
    }

    /// Returns the `SecretChat` of the specified id. Panics if the secret chat is not present.
    ///
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an id returned by TDLib, it should be expected that the
    /// relative `SecretChat` exists in the list.
    pub(crate) fn secret_chat(&self, secret_chat_id: i32) -> SecretChat {
        self.imp()
            .secret_chats
            .borrow()
            .get(&secret_chat_id)
            .expect("Failed to get expected SecretChat")
            .clone()
    }

    pub(crate) fn download_file(&self, file_id: i32, sender: SyncSender<File>) {
        let mut downloading_files = self.imp().downloading_files.borrow_mut();
        match downloading_files.entry(file_id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(sender);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![sender]);

                let client_id = self.client_id();
                spawn(clone!(@weak self as obj => async move {
                    let result = functions::download_file(file_id, 5, 0, 0, false, client_id).await;
                    match result {
                        Ok(enums::File::File(file)) => {
                            obj.handle_file_update(file);
                        }
                        Err(e) => {
                            log::warn!("Error downloading a file: {:?}", e);
                        }
                    }
                }));
            }
        }
    }

    pub(crate) fn select_chat(&self, chat_id: i64) {
        let imp = self.imp();
        let chat = self.chat_list().get(chat_id);
        imp.sidebar.set_selected_chat(Some(chat));
        imp.leaflet.navigate(adw::NavigationDirection::Forward);
    }

    pub(crate) fn handle_paste_action(&self) {
        self.imp().content.handle_paste_action();
    }

    pub(crate) fn begin_chats_search(&self) {
        let imp = self.imp();
        imp.leaflet.navigate(adw::NavigationDirection::Back);
        imp.sidebar.begin_chats_search();
    }

    fn handle_file_update(&self, file: File) {
        let mut downloading_files = self.imp().downloading_files.borrow_mut();
        if let Entry::Occupied(mut entry) = downloading_files.entry(file.id) {
            // Keep only the senders with which it was possible to send successfully.
            // It is indeed possible that the object that created the sender and receiver and
            // attached it to the default main context has been disposed in the meantime.
            // This is problematic if it is now tried to upgrade a weak reference of this object in
            // the receiver closure.
            // It will either panic directly if `@default-panic` is used or the sender will return
            // an error in the `SyncSender::send()` function if
            // `default-return glib::Continue(false)` is used. In the latter case, the Receiver
            // will be detached from the main context, which will cause the sending to fail.
            entry
                .get_mut()
                .retain(|sender| sender.send(file.clone()).is_ok());

            if file.local.is_downloading_completed || entry.get().is_empty() {
                entry.remove();
            }
        }
    }

    pub(crate) fn client_id(&self) -> i32 {
        self.imp().client_id.get()
    }

    pub(crate) fn database_info(&self) -> &BoxedDatabaseInfo {
        self.imp().database_info.get().unwrap()
    }

    pub(crate) fn me(&self) -> User {
        self.imp().me.upgrade().unwrap()
    }

    pub(crate) fn set_me(&self, me: &User) {
        let imp = self.imp();
        assert!(imp.me.upgrade().is_none());
        imp.me.set(Some(me));
        self.notify("me");
    }

    pub(crate) fn chat_list(&self) -> &ChatList {
        self.imp().chat_list.get_or_init(|| ChatList::new(self))
    }

    pub(crate) fn user_list(&self) -> &UserList {
        self.imp().user_list.get_or_init(|| UserList::new(self))
    }

    fn private_chats_notification_settings(&self) -> Option<BoxedScopeNotificationSettings> {
        self.imp()
            .private_chats_notification_settings
            .borrow()
            .clone()
    }

    fn set_private_chats_notification_settings(
        &self,
        settings: Option<BoxedScopeNotificationSettings>,
    ) {
        if self.private_chats_notification_settings() == settings {
            return;
        }
        self.imp()
            .private_chats_notification_settings
            .replace(settings);
        self.notify("private-chats-notification-settings")
    }

    fn group_chats_notification_settings(&self) -> Option<BoxedScopeNotificationSettings> {
        self.imp()
            .group_chats_notification_settings
            .borrow()
            .clone()
    }

    fn set_group_chats_notification_settings(
        &self,
        settings: Option<BoxedScopeNotificationSettings>,
    ) {
        if self.group_chats_notification_settings() == settings {
            return;
        }
        self.imp()
            .group_chats_notification_settings
            .replace(settings);
        self.notify("group-chats-notification-settings")
    }

    fn channel_chats_notification_settings(&self) -> Option<BoxedScopeNotificationSettings> {
        self.imp()
            .channel_chats_notification_settings
            .borrow()
            .clone()
    }

    fn set_channel_chats_notification_settings(
        &self,
        settings: Option<BoxedScopeNotificationSettings>,
    ) {
        if self.channel_chats_notification_settings() == settings {
            return;
        }
        self.imp()
            .channel_chats_notification_settings
            .replace(settings);
        self.notify("channel-chats-notification-settings")
    }

    pub(crate) fn fetch_chats(&self) {
        let client_id = self.imp().client_id.get();
        self.chat_list().fetch(client_id);
    }

    pub(crate) fn set_sessions(&self, sessions: &gtk::SelectionModel) {
        self.imp().sidebar.set_sessions(sessions, self);
    }
}
