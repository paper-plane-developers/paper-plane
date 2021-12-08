mod avatar;
mod basic_group;
mod basic_group_list;
mod chat;
mod chat_list;
mod components;
mod content;
mod secret_chat;
mod secret_chat_list;
mod sidebar;
mod supergroup;
mod supergroup_list;
mod user;
mod user_list;

use self::avatar::Avatar;
use self::basic_group::BasicGroup;
use self::basic_group_list::BasicGroupList;
pub use self::chat::{Chat, ChatType};
use self::chat_list::ChatList;
use self::content::Content;
use self::secret_chat::SecretChat;
use self::secret_chat_list::SecretChatList;
use self::sidebar::Sidebar;
use self::supergroup::Supergroup;
use self::supergroup_list::SupergroupList;
use self::user::User;
use self::user_list::UserList;

use glib::{clone, SyncSender};
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use std::collections::hash_map::{Entry, HashMap};
use tdgrand::enums::{NotificationSettingsScope, Update, User as TelegramUser};
use tdgrand::functions;
use tdgrand::types::{File, ScopeNotificationSettings};

use crate::utils::do_async;
use crate::RUNTIME;

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedScopeNotificationSettings")]
pub struct BoxedScopeNotificationSettings(pub Option<ScopeNotificationSettings>);

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/session.ui")]
    pub struct Session {
        pub client_id: Cell<i32>,
        pub me: RefCell<Option<User>>,
        pub chat_list: OnceCell<ChatList>,
        pub user_list: OnceCell<UserList>,
        pub basic_group_list: OnceCell<BasicGroupList>,
        pub supergroup_list: OnceCell<SupergroupList>,
        pub secret_chat_list: OnceCell<SecretChatList>,
        pub selected_chat: RefCell<Option<Chat>>,
        pub private_chats_notification_settings: RefCell<BoxedScopeNotificationSettings>,
        pub group_chats_notification_settings: RefCell<BoxedScopeNotificationSettings>,
        pub channel_chats_notification_settings: RefCell<BoxedScopeNotificationSettings>,
        pub downloading_files: RefCell<HashMap<i32, Vec<SyncSender<File>>>>,
        #[template_child]
        pub leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub sidebar: TemplateChild<Sidebar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Session {
        const NAME: &'static str = "Session";
        type Type = super::Session;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Content::static_type();
            Self::bind_template(klass);

            klass.install_action("content.go-back", None, move |widget, _, _| {
                let self_ = Self::from_instance(widget);
                self_.leaflet.navigate(adw::NavigationDirection::Back);
            });
            klass.install_action("session.log-out", None, move |widget, _, _| {
                widget.log_out();
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
                    glib::ParamSpec::new_int(
                        "client-id",
                        "Client Id",
                        "The client id",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "me",
                        "Me",
                        "The own user id of this session",
                        User::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "chat-list",
                        "Chat List",
                        "A list of chats",
                        ChatList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "user-list",
                        "User List",
                        "The list of users of this session",
                        ChatList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "basic-group-list",
                        "Basic Group List",
                        "The list of basic groups of this session",
                        BasicGroupList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "supergroup-list",
                        "Supergroup List",
                        "The list of supergroups of this session",
                        SupergroupList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "secret-chat-list",
                        "Secret Chat List",
                        "The list of secret chats of this session",
                        SecretChatList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-chat",
                        "Selected Chat",
                        "The selected chat in this sidebar",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "private-chats-notification-settings",
                        "Private Chats Notification Settings",
                        "This session's notification settings for private chats",
                        BoxedScopeNotificationSettings::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "group-chats-notification-settings",
                        "Group Chats Notification Settings",
                        "This session's notification settings for group chats",
                        BoxedScopeNotificationSettings::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boxed(
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
                "selected-chat" => {
                    let selected_chat = value.get().unwrap();
                    obj.set_selected_chat(selected_chat);
                }
                "private-chats-notification-settings" => {
                    let scope_notification_settings =
                        value.get::<BoxedScopeNotificationSettings>().unwrap();
                    obj.set_private_chats_notification_settings(scope_notification_settings);
                }
                "group-chats-notification-settings" => {
                    let scope_notification_settings =
                        value.get::<BoxedScopeNotificationSettings>().unwrap();
                    obj.set_group_chats_notification_settings(scope_notification_settings);
                }
                "channel-chats-notification-settings" => {
                    let scope_notification_settings =
                        value.get::<BoxedScopeNotificationSettings>().unwrap();
                    obj.set_channel_chats_notification_settings(scope_notification_settings);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "client-id" => obj.client_id().to_value(),
                "me" => obj.me().to_value(),
                "chat-list" => obj.chat_list().to_value(),
                "user-list" => obj.user_list().to_value(),
                "basic-group-list" => obj.basic_group_list().to_value(),
                "supergroup-list" => obj.supergroup_list().to_value(),
                "secret-chat-list" => obj.secret_chat_list().to_value(),
                "selected-chat" => obj.selected_chat().to_value(),
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

            self.leaflet.connect_child_transition_running_notify(
                clone!(@weak obj => move |leaflet| {
                    if !leaflet.is_child_transition_running()
                        && leaflet.visible_child().unwrap() == *Self::from_instance(&obj).sidebar {

                        // We deselect the chat when the transition to the sidebar is finished.
                        obj.set_selected_chat(None);
                    }
                }),
            );

            obj.fetch_me();
            obj.fetch_chats();
        }
    }

    impl WidgetImpl for Session {}
    impl BinImpl for Session {}
}

glib::wrapper! {
    pub struct Session(ObjectSubclass<imp::Session>)
        @extends gtk::Widget, adw::Bin;
}

impl Session {
    pub fn new(client_id: i32) -> Self {
        glib::Object::new(&[("client-id", &client_id)]).expect("Failed to create Session")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::NewMessage(_)
            | Update::MessageSendSucceeded(_)
            | Update::MessageContent(_)
            | Update::NewChat(_)
            | Update::ChatTitle(_)
            | Update::ChatPhoto(_)
            | Update::ChatLastMessage(_)
            | Update::ChatNotificationSettings(_)
            | Update::ChatPosition(_)
            | Update::ChatUnreadMentionCount(_)
            | Update::MessageMentionRead(_)
            | Update::ChatReadInbox(_)
            | Update::ChatDraftMessage(_)
            | Update::DeleteMessages(_) => {
                self.chat_list().handle_update(update);
            }
            Update::ScopeNotificationSettings(update) => {
                let settings = BoxedScopeNotificationSettings(Some(update.notification_settings));
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
            Update::User(_) | Update::UserStatus(_) => {
                self.user_list().handle_update(update);
            }
            Update::BasicGroup(_) => self.basic_group_list().handle_update(&update),
            Update::Supergroup(_) => self.supergroup_list().handle_update(&update),
            Update::SecretChat(_) => self.secret_chat_list().handle_update(&update),
            Update::File(update) => {
                self.handle_file_update(update.file);
            }
            _ => {}
        }
    }

    pub fn download_file(&self, file_id: i32, sender: SyncSender<File>) {
        let self_ = imp::Session::from_instance(self);

        let mut downloading_files = self_.downloading_files.borrow_mut();
        match downloading_files.entry(file_id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(sender);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![sender]);

                let client_id = self.client_id();
                RUNTIME.spawn(async move {
                    functions::DownloadFile::new()
                        .file_id(file_id)
                        .priority(5)
                        .send(client_id)
                        .await
                        .unwrap();
                });
            }
        }
    }

    pub fn begin_chats_search(&self) {
        let self_ = imp::Session::from_instance(self);
        self_.leaflet.navigate(adw::NavigationDirection::Back);
        self_.sidebar.begin_chats_search();
    }

    fn handle_file_update(&self, file: File) {
        let self_ = imp::Session::from_instance(self);

        let mut downloading_files = self_.downloading_files.borrow_mut();
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

    fn log_out(&self) {
        let client_id = self.client_id();
        RUNTIME.spawn(async move {
            functions::LogOut::new().send(client_id).await.unwrap();
        });
    }

    pub fn client_id(&self) -> i32 {
        let self_ = imp::Session::from_instance(self);
        self_.client_id.get()
    }

    pub fn me(&self) -> Option<User> {
        let self_ = imp::Session::from_instance(self);
        self_.me.borrow().clone()
    }

    pub fn chat_list(&self) -> &ChatList {
        let self_ = imp::Session::from_instance(self);
        self_.chat_list.get_or_init(|| ChatList::new(self))
    }

    pub fn user_list(&self) -> &UserList {
        let self_ = imp::Session::from_instance(self);
        self_.user_list.get_or_init(|| UserList::new(self))
    }

    pub fn basic_group_list(&self) -> &BasicGroupList {
        let self_ = imp::Session::from_instance(self);
        self_.basic_group_list.get_or_init(BasicGroupList::new)
    }

    pub fn supergroup_list(&self) -> &SupergroupList {
        let self_ = imp::Session::from_instance(self);
        self_.supergroup_list.get_or_init(SupergroupList::new)
    }

    pub fn secret_chat_list(&self) -> &SecretChatList {
        let self_ = imp::Session::from_instance(self);
        self_
            .secret_chat_list
            .get_or_init(|| SecretChatList::new(self))
    }

    fn selected_chat(&self) -> Option<Chat> {
        let self_ = imp::Session::from_instance(self);
        self_.selected_chat.borrow().clone()
    }

    fn set_selected_chat(&self, selected_chat: Option<Chat>) {
        if self.selected_chat() == selected_chat {
            return;
        }

        let self_ = imp::Session::from_instance(self);
        if selected_chat.is_some() {
            self_.leaflet.navigate(adw::NavigationDirection::Forward);
        }

        self_.selected_chat.replace(selected_chat);
        self.notify("selected-chat");
    }

    fn private_chats_notification_settings(&self) -> BoxedScopeNotificationSettings {
        let self_ = imp::Session::from_instance(self);
        self_.private_chats_notification_settings.borrow().clone()
    }

    fn set_private_chats_notification_settings(&self, settings: BoxedScopeNotificationSettings) {
        if self.private_chats_notification_settings().0 == settings.0 {
            return;
        }
        let self_ = imp::Session::from_instance(self);
        self_.private_chats_notification_settings.replace(settings);
        self.notify("private-chats-notification-settings")
    }

    fn group_chats_notification_settings(&self) -> BoxedScopeNotificationSettings {
        let self_ = imp::Session::from_instance(self);
        self_.group_chats_notification_settings.borrow().clone()
    }

    fn set_group_chats_notification_settings(&self, settings: BoxedScopeNotificationSettings) {
        if self.group_chats_notification_settings().0 == settings.0 {
            return;
        }
        let self_ = imp::Session::from_instance(self);
        self_.group_chats_notification_settings.replace(settings);
        self.notify("group-chats-notification-settings")
    }

    fn channel_chats_notification_settings(&self) -> BoxedScopeNotificationSettings {
        let self_ = imp::Session::from_instance(self);
        self_.channel_chats_notification_settings.borrow().clone()
    }

    fn set_channel_chats_notification_settings(&self, settings: BoxedScopeNotificationSettings) {
        if self.channel_chats_notification_settings().0 == settings.0 {
            return;
        }
        let self_ = imp::Session::from_instance(self);
        self_.channel_chats_notification_settings.replace(settings);
        self.notify("channel-chats-notification-settings")
    }

    fn fetch_me(&self) {
        let client_id = self.client_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move { functions::GetMe::new().send(client_id).await },
            clone!(@weak self as obj => move |result| async move {
                let TelegramUser::User(me) = result.unwrap();

                let me = User::from_td_object(me, &obj);
                obj.user_list().insert_user(me.clone());

                imp::Session::from_instance(&obj)
                    .me
                    .replace(Some(me));

                obj.notify("me");
            }),
        );
    }

    fn fetch_chats(&self) {
        let self_ = imp::Session::from_instance(self);
        let client_id = self_.client_id.get();
        self.chat_list().fetch(client_id);
    }
}
