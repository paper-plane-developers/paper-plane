mod avatar;
mod basic_group;
mod chat;
mod chat_action;
mod chat_action_list;
mod chat_folder_list;
mod chat_history_item;
mod chat_history_model;
mod chat_list;
mod chat_list_item;
mod client;
mod client_manager;
mod client_state_auth;
mod client_state_auth_code;
mod client_state_auth_other_device;
mod client_state_auth_password;
mod client_state_auth_phone_number;
mod client_state_auth_registration;
mod client_state_logging_out;
mod client_state_session;
mod country_info;
mod country_list;
mod message;
mod message_forward_info;
mod message_interaction_info;
mod secret_chat;
mod sponsored_message;
mod supergroup;
mod user;

use gtk::glib;
use tdlib::enums::BlockList;
use tdlib::enums::ChatMemberStatus;
use tdlib::enums::MessageContent;
use tdlib::enums::MessageReplyTo;
use tdlib::enums::MessageSendingState;
use tdlib::enums::UserStatus;
use tdlib::enums::UserType;
use tdlib::types::AuthorizationStateWaitCode;
use tdlib::types::AuthorizationStateWaitOtherDeviceConfirmation;
use tdlib::types::AuthorizationStateWaitPassword;
use tdlib::types::AuthorizationStateWaitRegistration;
use tdlib::types::ChatNotificationSettings;
use tdlib::types::ChatPermissions;
use tdlib::types::DraftMessage;
use tdlib::types::FormattedText;
use tdlib::types::ScopeNotificationSettings;
use tdlib::types::UpdateNotificationGroup;

pub(crate) use self::avatar::Avatar;
pub(crate) use self::basic_group::BasicGroup;
pub(crate) use self::chat::Chat;
pub(crate) use self::chat::ChatType;
pub(crate) use self::chat_action::ChatAction;
pub(crate) use self::chat_action_list::ChatActionList;
pub(crate) use self::chat_folder_list::ChatFolderList;
pub(crate) use self::chat_history_item::ChatHistoryItem;
pub(crate) use self::chat_history_item::ChatHistoryItemType;
pub(crate) use self::chat_history_model::ChatHistoryError;
pub(crate) use self::chat_history_model::ChatHistoryModel;
pub(crate) use self::chat_list::ChatList;
pub(crate) use self::chat_list_item::ChatListItem;
pub(crate) use self::client::Client;
pub(crate) use self::client::DatabaseInfo;
pub(crate) use self::client_manager::ClientManager;
pub(crate) use self::client_state_auth::ClientStateAuth;
pub(crate) use self::client_state_auth_code::ClientStateAuthCode;
pub(crate) use self::client_state_auth_other_device::ClientStateAuthOtherDevice;
pub(crate) use self::client_state_auth_password::ClientStateAuthPassword;
pub(crate) use self::client_state_auth_password::SendPasswordRecoveryCodeResult;
pub(crate) use self::client_state_auth_phone_number::ClientStateAuthPhoneNumber;
pub(crate) use self::client_state_auth_phone_number::SendPhoneNumberResult;
pub(crate) use self::client_state_auth_registration::ClientStateAuthRegistration;
pub(crate) use self::client_state_logging_out::ClientStateLoggingOut;
pub(crate) use self::client_state_session::ClientStateSession;
pub(crate) use self::country_info::CountryInfo;
pub(crate) use self::country_list::CountryList;
pub(crate) use self::message::Message;
pub(crate) use self::message::MessageSender;
pub(crate) use self::message_forward_info::MessageForwardInfo;
pub(crate) use self::message_forward_info::MessageForwardOrigin;
pub(crate) use self::message_interaction_info::MessageInteractionInfo;
pub(crate) use self::secret_chat::SecretChat;
pub(crate) use self::secret_chat::SecretChatState;
pub(crate) use self::sponsored_message::SponsoredMessage;
pub(crate) use self::supergroup::Supergroup;
pub(crate) use self::user::User;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedChatListType")]
pub(crate) struct BoxedChatListType(pub(crate) tdlib::enums::ChatList);
impl Default for BoxedChatListType {
    fn default() -> Self {
        Self(tdlib::enums::ChatList::Main)
    }
}
impl BoxedChatListType {
    pub(crate) fn chat_folder_id(&self) -> Option<i32> {
        match &self.0 {
            tdlib::enums::ChatList::Folder(chat_list_folder) => {
                Some(chat_list_folder.chat_folder_id)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedUpdateNotificationGroup")]
pub(crate) struct BoxedUpdateNotificationGroup(pub(crate) UpdateNotificationGroup);

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedAuthorizationStateWaitOtherDeviceConfirmation", nullable)]
pub(crate) struct BoxedAuthorizationStateWaitOtherDeviceConfirmation(
    pub(crate) AuthorizationStateWaitOtherDeviceConfirmation,
);

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedAuthorizationStateWaitCode", nullable)]
pub(crate) struct BoxedAuthorizationStateWaitCode(pub(crate) AuthorizationStateWaitCode);

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedAuthorizationStateWaitRegistration")]
pub(crate) struct BoxedAuthorizationStateWaitRegistration(
    pub(crate) AuthorizationStateWaitRegistration,
);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedAuthorizationStateWaitPassword", nullable)]
pub(crate) struct BoxedAuthorizationStateWaitPassword(pub(crate) AuthorizationStateWaitPassword);

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedDatabaseInfo", nullable)]
pub(crate) struct BoxedDatabaseInfo(pub(crate) DatabaseInfo);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedChatMemberStatus")]
pub(crate) struct BoxedChatMemberStatus(pub(crate) ChatMemberStatus);
impl Default for BoxedChatMemberStatus {
    fn default() -> Self {
        Self(ChatMemberStatus::Member)
    }
}

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedChatActionType")]
pub(crate) struct BoxedChatActionType(pub(crate) tdlib::enums::ChatAction);

#[derive(Clone, Debug, Default, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedChatNotificationSettings")]
pub(crate) struct BoxedChatNotificationSettings(pub(crate) ChatNotificationSettings);

#[derive(Clone, Debug, Default, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedChatPermissions")]
pub(crate) struct BoxedChatPermissions(pub(crate) ChatPermissions);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedDraftMessage", nullable)]
pub(crate) struct BoxedDraftMessage(pub(crate) DraftMessage);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedBlockList", nullable)]
pub(crate) struct BoxedBlockList(pub(crate) BlockList);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedFormattedText", nullable)]
pub(crate) struct BoxedFormattedText(pub(crate) FormattedText);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedMessageContent")]
pub(crate) struct BoxedMessageContent(pub(crate) MessageContent);
impl Default for BoxedMessageContent {
    fn default() -> Self {
        Self(MessageContent::MessageUnsupported)
    }
}

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedMessageReplyTo", nullable)]
pub(crate) struct BoxedMessageReplyTo(pub(crate) MessageReplyTo);

#[derive(Clone, Debug, Default, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedScopeNotificationSettings", nullable)]
pub(crate) struct BoxedScopeNotificationSettings(pub(crate) ScopeNotificationSettings);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserStatus")]
pub(crate) struct BoxedUserStatus(pub(crate) UserStatus);
impl Default for BoxedUserStatus {
    fn default() -> Self {
        Self(UserStatus::Empty)
    }
}

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserType")]
pub(crate) struct BoxedUserType(pub(crate) UserType);
impl Default for BoxedUserType {
    fn default() -> Self {
        Self(UserType::Unknown)
    }
}

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedMessageSendingState", nullable)]
pub(crate) struct BoxedMessageSendingState(pub(crate) MessageSendingState);
