mod avatar;
mod basic_group;
mod chat;
mod chat_action;
mod chat_action_list;
mod chat_list;
mod chat_list_item;
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
use tdlib::enums::ChatMemberStatus;
use tdlib::enums::MessageContent;
use tdlib::enums::MessageSendingState;
use tdlib::enums::UserStatus;
use tdlib::enums::UserType;
use tdlib::types::ChatNotificationSettings;
use tdlib::types::ChatPermissions;
use tdlib::types::DraftMessage;
use tdlib::types::FormattedText;
use tdlib::types::ScopeNotificationSettings;

pub(crate) use self::avatar::Avatar;
pub(crate) use self::basic_group::BasicGroup;
pub(crate) use self::chat::Chat;
pub(crate) use self::chat::ChatType;
pub(crate) use self::chat_action::ChatAction;
pub(crate) use self::chat_action_list::ChatActionList;
pub(crate) use self::chat_list::ChatList;
pub(crate) use self::chat_list_item::ChatListItem;
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

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedMessageSendingState", nullable)]
pub(crate) struct BoxedMessageSendingState(pub(crate) MessageSendingState);
