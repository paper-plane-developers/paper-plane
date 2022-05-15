mod avatar;
mod basic_group;
mod basic_group_list;
mod chat;
mod chat_action;
mod chat_action_list;
mod chat_history;
mod chat_list;
mod country_info;
mod country_list;
mod item;
mod message;
mod message_forward_info;
mod secret_chat;
mod secret_chat_list;
mod sponsored_message;
mod supergroup;
mod supergroup_list;
mod user;
mod user_list;

pub(crate) use self::avatar::Avatar;
pub(crate) use self::basic_group::BasicGroup;
pub(crate) use self::basic_group_list::BasicGroupList;
pub(crate) use self::chat::{Chat, ChatType};
pub(crate) use self::chat_action::ChatAction;
pub(crate) use self::chat_action_list::ChatActionList;
use self::chat_history::ChatHistory;
pub(crate) use self::chat_list::ChatList;
pub(crate) use self::country_info::CountryInfo;
pub(crate) use self::country_list::CountryList;
pub(crate) use self::item::{Item, ItemType};
pub(crate) use self::message::{Message, MessageSender};
pub(crate) use self::message_forward_info::{MessageForwardInfo, MessageForwardOrigin};
use self::secret_chat::SecretChat;
pub(crate) use self::secret_chat_list::SecretChatList;
pub(crate) use self::sponsored_message::SponsoredMessage;
pub(crate) use self::supergroup::Supergroup;
pub(crate) use self::supergroup_list::SupergroupList;
pub(crate) use self::user::User;
pub(crate) use self::user_list::UserList;

use gtk::glib;
use tdlib::enums::{ChatMemberStatus, MessageContent, UserStatus, UserType};
use tdlib::types::{
    ChatNotificationSettings, ChatPermissions, DraftMessage, FormattedText,
    ScopeNotificationSettings,
};

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedChatMemberStatus")]
pub(crate) struct BoxedChatMemberStatus(pub(crate) ChatMemberStatus);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedChatNotificationSettings")]
pub(crate) struct BoxedChatNotificationSettings(pub(crate) ChatNotificationSettings);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedChatPermissions")]
pub(crate) struct BoxedChatPermissions(pub(crate) ChatPermissions);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedDraftMessage", nullable)]
pub(crate) struct BoxedDraftMessage(pub(crate) DraftMessage);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedFormattedText", nullable)]
pub(crate) struct BoxedFormattedText(pub(crate) FormattedText);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedMessageContent")]
pub(crate) struct BoxedMessageContent(pub(crate) MessageContent);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedScopeNotificationSettings", nullable)]
pub(crate) struct BoxedScopeNotificationSettings(pub(crate) ScopeNotificationSettings);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserStatus")]
pub(crate) struct BoxedUserStatus(pub(crate) UserStatus);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserType")]
pub(crate) struct BoxedUserType(pub(crate) UserType);
