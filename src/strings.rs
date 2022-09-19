use ellipse::Ellipse;
use gettextrs::gettext;
use gtk::glib;
use tdlib::enums::{CallDiscardReason, UserStatus, UserType};

use crate::i18n::{gettext_f, ngettext_f};
use crate::tdlib::{Chat, ChatType, Message, MessageSender, User};
use crate::utils::{freplace, human_friendly_duration};

fn user_display_name(user: &User) -> String {
    if let UserType::Deleted = user.type_().0 {
        gettext("Deleted Account")
    } else if user.last_name().is_empty() {
        user.first_name()
    } else if user.first_name().is_empty() {
        user.last_name()
    } else {
        user.first_name() + " " + &user.last_name()
    }
}

pub(crate) fn user_status(status: &UserStatus) -> String {
    match status {
        UserStatus::Empty => gettext("last seen a long time ago"),
        UserStatus::Online(_) => gettext("online"),
        UserStatus::Offline(data) => {
            let now = glib::DateTime::now_local().unwrap();
            let was_online = glib::DateTime::from_unix_local(data.was_online as i64).unwrap();
            let time_span = now.difference(&was_online);

            // TODO: Add a way to update the string when time passes
            if time_span.as_days() > 1 {
                // Translators: This is an online status with the date
                was_online.format(&gettext("last seen %x")).unwrap().into()
            } else if now.day_of_week() != was_online.day_of_week() && now.hour() >= 1 {
                // Translators: This is an online status with the time without seconds
                was_online
                    .format(&gettext("last seen yesterday at %l:%M %p"))
                    .unwrap()
                    .into()
            } else if time_span.as_hours() > 0 {
                ngettext_f(
                    "last seen {num} hour ago",
                    "last seen {num} hours ago",
                    time_span.as_hours() as u32,
                    &[("num", &time_span.as_hours().to_string())],
                )
            } else if time_span.as_minutes() > 0 {
                ngettext_f(
                    "last seen {num} minute ago",
                    "last seen {num} minutes ago",
                    time_span.as_minutes() as u32,
                    &[("num", &time_span.as_minutes().to_string())],
                )
            } else {
                gettext("last seen just now")
            }
        }
        UserStatus::Recently => gettext("last seen recently"),
        UserStatus::LastWeek => gettext("last seen within a week"),
        UserStatus::LastMonth => gettext("last seen within a month"),
    }
}

pub(crate) fn message_sender(sender: &MessageSender) -> String {
    match sender {
        MessageSender::Chat(chat) => chat.title(),
        MessageSender::User(user) => user_display_name(user),
    }
}

pub(crate) fn message_content(message: &Message) -> String {
    use tdlib::enums::MessageContent::*;
    let sender = message.sender();
    let chat = message.chat();

    match message.content().0 {
        MessageText(data) => data.text.text,
        MessageAnimation(data) => message_animation(&data.caption.text),
        MessageAudio(data) => {
            message_audio(&data.audio.title, &data.audio.performer, &data.caption.text)
        }
        MessageDocument(data) => message_document(&data.document.file_name, &data.caption.text),
        MessagePhoto(data) => message_photo(&data.caption.text),
        MessageExpiredPhoto => gettext("Photo has expired"),
        MessageSticker(data) => message_sticker(&data.sticker.emoji),
        MessageVideo(data) => message_video(&data.caption.text),
        MessageExpiredVideo => gettext("Video has expired"),
        MessageVideoNote(_) => gettext("Video Message"),
        MessageVoiceNote(data) => message_voice_note(&data.caption.text),
        MessageCall(data) => message_call(
            &data.discard_reason,
            data.is_video,
            data.duration,
            message.is_outgoing(),
        ),
        MessageBasicGroupChatCreate(data) => message_basic_group_chat_create(&data.title, sender),
        MessageSupergroupChatCreate(data) => {
            message_supergroup_chat_create(&data.title, &chat, sender)
        }
        MessageChatChangeTitle(data) => message_chat_change_title(&data.title, &chat, sender),
        MessageChatChangePhoto(_) => message_chat_change_photo(&chat, sender),
        MessageChatDeletePhoto => message_chat_delete_photo(&chat, sender),
        MessageChatAddMembers(data) => {
            let added_users = data
                .member_user_ids
                .into_iter()
                .map(|id| chat.session().user_list().get(id))
                .collect();
            message_chat_add_members(sender, &added_users)
        }
        MessageChatJoinByLink => message_chat_join_by_link(sender),
        MessageChatJoinByRequest => message_chat_join_by_request(sender),
        MessageChatDeleteMember(data) => {
            let deleted_user = chat.session().user_list().get(data.user_id);
            message_chat_delete_member(&deleted_user, sender)
        }
        MessagePinMessage(data) => message_pin_message(data.message_id, &chat, sender),
        MessageContactRegistered => message_contact_registered(sender),
        _ => gettext("Unsupported Message"),
    }
}

fn message_animation(caption: &str) -> String {
    if caption.is_empty() {
        gettext("GIF")
    } else {
        gettext_f("GIF, {caption}", &[("caption", caption)])
    }
}

fn message_audio(title: &str, performer: &str, caption: &str) -> String {
    if caption.is_empty() {
        // Translators: This is an audio with the title and performer
        gettext_f(
            "{title} – {performer}",
            &[("title", title), ("performer", performer)],
        )
    } else {
        // Translators: This is an audio with the caption
        gettext_f(
            "{title} – {performer}, {caption}",
            &[
                ("title", title),
                ("performer", performer),
                ("caption", caption),
            ],
        )
    }
}

fn message_document(file_name: &str, caption: &str) -> String {
    if caption.is_empty() {
        file_name.into()
    } else {
        // Translators: This is a file with the caption
        gettext_f("{file_name}, {caption}", &[("caption", caption)])
    }
}

fn message_photo(caption: &str) -> String {
    if caption.is_empty() {
        gettext("Photo")
    } else {
        gettext_f("Photo, {caption}", &[("caption", caption)])
    }
}

fn message_sticker(emoji: &str) -> String {
    // Translators: This is a sticker with the associated emoji
    gettext_f("{emoji} Sticker", &[("emoji", emoji)])
}

fn message_video(caption: &str) -> String {
    if caption.is_empty() {
        gettext("Video")
    } else {
        gettext_f("Video, {caption}", &[("caption", caption)])
    }
}

fn message_voice_note(caption: &str) -> String {
    if caption.is_empty() {
        gettext("Voice Message")
    } else {
        gettext_f("Voice Message, {caption}", &[("caption", caption)])
    }
}

fn message_call(
    discard_reason: &CallDiscardReason,
    is_video: bool,
    duration: i32,
    is_outgoing: bool,
) -> String {
    match discard_reason {
        CallDiscardReason::Declined => {
            if is_outgoing {
                // Telegram Desktop/Android labels declined outgoing calls just as
                // "Outgoing call" and puts a red arrow in the message bubble. We should be
                // more accurate here.
                if is_video {
                    gettext("Declined outgoing video call")
                } else {
                    gettext("Declined outgoing call")
                }
            // Telegram Android labels declined incoming calls as "Incoming call". Telegram
            // Desktop labels it as "Declined call" and is a bit inconsistent with outgoing
            // calls ^.
            } else if is_video {
                gettext("Declined incoming video call")
            } else {
                gettext("Declined incoming call")
            }
        }
        CallDiscardReason::Disconnected | CallDiscardReason::HungUp | CallDiscardReason::Empty => {
            made_message_call(is_outgoing, is_video, duration)
        }
        CallDiscardReason::Missed => {
            if is_outgoing {
                gettext("Canceled call")
            } else {
                gettext("Missed call")
            }
        }
    }
}

/// This method returns the text for all calls that have actually been made.
/// This means that the called party has accepted the call.
fn made_message_call(is_outgoing: bool, is_video: bool, duration: i32) -> String {
    if is_outgoing {
        if duration > 0 {
            if is_video {
                gettext_f(
                    "Outgoing video call ({duration})",
                    &[("duration", &human_friendly_duration(duration))],
                )
            } else {
                gettext_f(
                    "Outgoing call ({duration})",
                    &[("duration", &human_friendly_duration(duration))],
                )
            }
        } else if is_video {
            gettext("Outgoing video call")
        } else {
            gettext("Outgoing call")
        }
    } else if duration > 0 {
        if is_video {
            gettext_f(
                "Incoming video call ({duration})",
                &[("duration", &human_friendly_duration(duration))],
            )
        } else {
            gettext_f(
                "Incoming call ({duration})",
                &[("duration", &human_friendly_duration(duration))],
            )
        }
    } else if is_video {
        gettext("Incoming video call")
    } else {
        gettext("Incoming call")
    }
}

fn message_basic_group_chat_create(title: &str, sender: &MessageSender) -> String {
    let sender = message_sender(sender);
    gettext_f(
        "{sender} created the group \"{title}\"",
        &[("sender", &sender), ("title", title)],
    )
}

fn message_supergroup_chat_create(title: &str, chat: &Chat, sender: &MessageSender) -> String {
    match chat.type_() {
        ChatType::Supergroup(supergroup) if supergroup.is_channel() => gettext("Channel created"),
        _ => {
            let sender = message_sender(sender);
            gettext_f(
                "{sender} created the group \"{title}\"",
                &[("sender", &sender), ("title", title)],
            )
        }
    }
}

fn message_chat_change_title(title: &str, chat: &Chat, sender: &MessageSender) -> String {
    match chat.type_() {
        ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
            gettext_f("Channel renamed to \"{title}\"", &[("title", title)])
        }
        _ => {
            let sender = message_sender(sender);
            gettext_f(
                "{sender} changed the group name to \"{title}\"",
                &[("sender", &sender), ("title", title)],
            )
        }
    }
}

fn message_chat_change_photo(chat: &Chat, sender: &MessageSender) -> String {
    match chat.type_() {
        ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
            gettext("Channel photo changed")
        }
        _ => {
            let sender = message_sender(sender);
            gettext_f("{sender} changed the group photo", &[("sender", &sender)])
        }
    }
}

fn message_chat_delete_photo(chat: &Chat, sender: &MessageSender) -> String {
    match chat.type_() {
        ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
            gettext("Channel photo removed")
        }
        _ => {
            let sender = message_sender(sender);
            gettext_f("{sender} removed the group photo", &[("sender", &sender)])
        }
    }
}

fn message_chat_add_members(sender: &MessageSender, added_users: &Vec<User>) -> String {
    let sender_string = message_sender(sender);
    if sender.as_user().map(User::id) == added_users.first().map(User::id) {
        gettext_f("{sender} joined the group", &[("sender", &sender_string)])
    } else if added_users.len() == 2 {
        let first_user = user_display_name(added_users.first().unwrap());
        let second_user = user_display_name(added_users.last().unwrap());
        gettext_f(
            "{sender} added {first_user} and {second_user}",
            &[
                ("sender", &sender_string),
                ("first_user", &first_user),
                ("second_user", &second_user),
            ],
        )
    } else {
        let users = added_users
            .iter()
            .map(user_display_name)
            .collect::<Vec<_>>()
            .join(", ");
        gettext_f(
            "{sender} added {users}",
            &[("sender", &sender_string), ("users", &users)],
        )
    }
}

fn message_chat_join_by_link(sender: &MessageSender) -> String {
    let sender = message_sender(sender);
    gettext_f(
        "{sender} joined the group via invite link",
        &[("sender", &sender)],
    )
}

fn message_chat_join_by_request(sender: &MessageSender) -> String {
    let sender = message_sender(sender);
    gettext_f("{sender} joined the group", &[("sender", &sender)])
}

fn message_chat_delete_member(deleted_user: &User, sender: &MessageSender) -> String {
    let sender_string = message_sender(sender);
    match sender {
        MessageSender::User(user) if user.id() == deleted_user.id() => {
            gettext_f("{sender} left the group", &[("sender", &sender_string)])
        }
        _ => {
            let deleted_user_name = user_display_name(deleted_user);
            gettext_f(
                "{sender} removed {user}",
                &[("sender", &sender_string), ("user", &deleted_user_name)],
            )
        }
    }
}

fn message_pin_message(message_id: i64, chat: &Chat, sender: &MessageSender) -> String {
    use tdlib::enums::MessageContent::*;

    // TODO: Add a way to retrieve the message and update the string
    // in case we don't have the message stored locally.
    let string = match chat.history().message_by_id(message_id) {
        Some(message) => match message.content().0 {
            MessageText(data) => {
                const TEXT_LENGTH: usize = 32;
                let text = data
                    .text
                    .text
                    .as_str()
                    .truncate_ellipse_with(TEXT_LENGTH, "…");

                gettext_f("{sender} pinned \"{text}\"", &[("text", &text)])
            }
            MessageAnimation(_) => gettext("{sender} pinned a GIF"),
            MessageAudio(_) => gettext("{sender} pinned an audio file"),
            MessageDocument(_) => gettext("{sender} pinned a file"),
            MessagePhoto(_) | MessageExpiredPhoto => gettext("{sender} pinned a photo"),
            MessageSticker(_) => gettext("{sender} pinned a sticker"),
            MessageVideo(_) | MessageExpiredVideo => gettext("{sender} pinned a video"),
            MessageVideoNote(_) => gettext("{sender} pinned a video message"),
            MessageVoiceNote(_) => gettext("{sender} pinned a voice message"),
            _ => gettext("{sender} pinned a message"),
        },
        None => gettext("{sender} pinned a message"),
    };

    let sender = message_sender(sender);
    freplace(string, &[("sender", &sender)])
}

fn message_contact_registered(sender: &MessageSender) -> String {
    let sender = message_sender(sender);
    gettext_f("{sender} joined Telegram", &[("sender", &sender)])
}
