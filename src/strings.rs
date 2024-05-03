use ellipse::Ellipse;
use gettextrs::gettext;
use gtk::glib;

use crate::i18n::gettext_f;
use crate::i18n::ngettext_f;
use crate::model;
use crate::types::MessageId;
use crate::utils;

pub(crate) fn chat_action(action: &model::ChatAction) -> String {
    use tdlib::enums::ChatAction::*;

    let chat = action.chat_();

    let show_sender = matches!(
        chat.chat_type(),
        model::ChatType::BasicGroup(_) | model::ChatType::Supergroup(_)
    );

    let td_action = &action.action_type().0;

    let action_group = chat.actions().group(td_action);

    match td_action {
        ChoosingContact => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is choosing a contact",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are choosing contacts",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are choosing contacts",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext("choosing a contact")
            }
        }
        ChoosingLocation => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is choosing a location",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are choosing locations",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are choosing locations",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext("choosing a location")
            }
        }
        ChoosingSticker => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is choosing a sticker",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are choosing stickers",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are choosing stickers",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext("choosing a sticker")
            }
        }
        RecordingVideo => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is recording a video",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are recording videos",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are recording videos",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext("recording a video")
            }
        }
        RecordingVideoNote => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is recording a video note",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are recording video notes",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are recording video notes",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext("recording a video note")
            }
        }
        RecordingVoiceNote => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is recording a voice note",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are recording voice notes",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are recording voice notes",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext("recording a voice note")
            }
        }
        StartPlayingGame => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is playing a game",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are playing games",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are playing games",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext("playing a game")
            }
        }
        Typing => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is typing",
                        &[("sender", &message_sender(&action_group[0].sender(), false))],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are typing",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f("{num} people are typing", &[("num", &len.to_string())]),
                }
            } else {
                gettext("typing")
            }
        }
        UploadingDocument(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is uploading a document ({progress}%)",
                        &[
                            ("sender", &message_sender(&action_group[0].sender(), false)),
                            ("progress", &action.progress.to_string()),
                        ],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are uploading documents",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are uploading documents",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext_f(
                    "uploading a document ({progress}%)",
                    &[("progress", &action.progress.to_string())],
                )
            }
        }
        UploadingPhoto(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is uploading a photo ({progress}%)",
                        &[
                            ("sender", &message_sender(&action_group[0].sender(), false)),
                            ("progress", &action.progress.to_string()),
                        ],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are uploading photos",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are uploading photos",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext_f(
                    "uploading a photo ({progress}%)",
                    &[("progress", &action.progress.to_string())],
                )
            }
        }
        UploadingVideo(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is uploading a video ({progress}%)",
                        &[
                            ("sender", &message_sender(&action_group[0].sender(), false)),
                            ("progress", &action.progress.to_string()),
                        ],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are uploading videos",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are uploading videos",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext_f(
                    "uploading a video ({progress}%)",
                    &[("progress", &action.progress.to_string())],
                )
            }
        }
        UploadingVideoNote(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is uploading a video note ({progress}%)",
                        &[
                            ("sender", &message_sender(&action_group[0].sender(), false)),
                            ("progress", &action.progress.to_string()),
                        ],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are uploading video notes",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are uploading video notes",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext_f(
                    "uploading a video note ({progress}%)",
                    &[("progress", &action.progress.to_string())],
                )
            }
        }
        UploadingVoiceNote(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is uploading a voice note ({progress}%)",
                        &[
                            ("sender", &message_sender(&action_group[0].sender(), false)),
                            ("progress", &action.progress.to_string()),
                        ],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are uploading voice notes",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are uploading voice notes",
                        &[("num", &len.to_string())],
                    ),
                }
            } else {
                gettext_f(
                    "uploading a voice note ({progress}%)",
                    &[("progress", &action.progress.to_string())],
                )
            }
        }
        WatchingAnimations(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext_f(
                        "{sender} is watching an animation {emoji}",
                        &[
                            ("sender", &message_sender(&action_group[0].sender(), false)),
                            ("emoji", &action.emoji),
                        ],
                    ),
                    2 => gettext_f(
                        "{sender1} and {sender2} are watching animations {emoji}",
                        &[
                            ("sender1", &message_sender(&action_group[0].sender(), false)),
                            ("sender2", &message_sender(&action_group[1].sender(), false)),
                            ("emoji", &action.emoji),
                        ],
                    ),
                    len => gettext_f(
                        "{num} people are watching animations {emoji}",
                        &[("num", &len.to_string()), ("emoji", &action.emoji)],
                    ),
                }
            } else {
                gettext("watching an animation")
            }
        }
        Cancel => unreachable!(),
    }
}

pub(crate) fn user_display_name(user: &model::User, use_full_name: bool) -> String {
    if let tdlib::enums::UserType::Deleted = user.user_type().0 {
        gettext("Deleted Account")
    } else if user.last_name().is_empty() || !use_full_name {
        user.first_name()
    } else if user.first_name().is_empty() {
        user.last_name()
    } else {
        user.first_name() + " " + &user.last_name()
    }
}

pub(crate) fn user_status(status: &tdlib::enums::UserStatus) -> String {
    use tdlib::enums::UserStatus::*;

    match status {
        Empty => gettext("last seen a long time ago"),
        Online(_) => gettext("online"),
        Offline(data) => {
            let now = glib::DateTime::now_local().unwrap();
            let was_online = glib::DateTime::from_unix_local(data.was_online as i64).unwrap();
            let time_span = now.difference(&was_online);

            // TODO: Add a way to update the string when time passes
            if time_span.as_days() > 1 {
                // Translators: This is an online status with the date
                was_online.format(&gettext("last seen %x")).unwrap().into()
            } else if now.day_of_week() != was_online.day_of_week() && now.hour() >= 1 {
                // Translators: This is an online status with the last seen time, without seconds
                // Here you may want to change to a 24-hours representation, based on your locale.
                // You can use this site to learn more: https://www.strfti.me/
                let format = gettext("last seen yesterday at %l:%M %p");
                was_online.format(&format).unwrap().into()
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
        Recently => gettext("last seen recently"),
        LastWeek => gettext("last seen within a week"),
        LastMonth => gettext("last seen within a month"),
    }
}

pub(crate) fn message_sender(sender: &model::MessageSender, use_full_name: bool) -> String {
    match sender {
        model::MessageSender::Chat(chat) => chat.title(),
        model::MessageSender::User(user) => user_display_name(user, use_full_name),
    }
}

pub(crate) fn message_content(message: &model::Message) -> String {
    use tdlib::enums::MessageContent::*;

    let sender = message.sender();
    let chat = message.chat_();

    match message.content().0 {
        MessageText(data) => data.text.text,
        MessageLocation(data) => {
            if data.live_period > 0 {
                gettext("Live Location")
            } else {
                gettext("Location")
            }
        }
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
        MessageAnimatedEmoji(data) => data.emoji,
        MessageDice(data) => data.emoji,
        MessageCall(data) => message_call(
            &data.discard_reason,
            data.is_video,
            data.duration,
            message.is_outgoing(),
        ),
        MessageBasicGroupChatCreate(data) => message_basic_group_chat_create(&data.title, &sender),
        MessageSupergroupChatCreate(data) => {
            message_supergroup_chat_create(&data.title, &chat, &sender)
        }
        MessageChatChangeTitle(data) => message_chat_change_title(&data.title, &chat, &sender),
        MessageChatChangePhoto(_) => message_chat_change_photo(&chat, &sender),
        MessageChatDeletePhoto => message_chat_delete_photo(&chat, &sender),
        MessageChatAddMembers(data) => {
            let added_users = data
                .member_user_ids
                .into_iter()
                .map(|id| chat.session_().user(id))
                .collect::<Vec<_>>();
            message_chat_add_members(&sender, &added_users)
        }
        MessageChatJoinByLink => message_chat_join_by_link(&sender),
        MessageChatJoinByRequest => message_chat_join_by_request(&sender),
        MessageChatDeleteMember(data) => {
            let deleted_user = chat.session_().user(data.user_id);
            message_chat_delete_member(&deleted_user, &sender)
        }
        MessagePinMessage(data) => message_pin_message(data.message_id, &chat, &sender),
        MessageScreenshotTaken => gettext_f(
            "{sender} took a screenshot!",
            &[("sender", &message_sender(&sender, true))],
        ),
        MessageGameScore(data) => message_game_score(&data, &chat, &sender),
        MessageContactRegistered => message_contact_registered(&sender),
        MessageVenue(data) => gettext_f("Location, {venue}", &[("venue", &data.venue.title)]),
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
        gettext_f(
            "{file_name}, {caption}",
            &[("file_name", file_name), ("caption", caption)],
        )
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
    discard_reason: &tdlib::enums::CallDiscardReason,
    is_video: bool,
    duration: i32,
    is_outgoing: bool,
) -> String {
    use tdlib::enums::CallDiscardReason::*;

    match discard_reason {
        Declined => {
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
        Disconnected | HungUp | Empty => made_message_call(is_outgoing, is_video, duration),
        Missed => {
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
                    &[("duration", &utils::human_friendly_duration(duration))],
                )
            } else {
                gettext_f(
                    "Outgoing call ({duration})",
                    &[("duration", &utils::human_friendly_duration(duration))],
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
                &[("duration", &utils::human_friendly_duration(duration))],
            )
        } else {
            gettext_f(
                "Incoming call ({duration})",
                &[("duration", &utils::human_friendly_duration(duration))],
            )
        }
    } else if is_video {
        gettext("Incoming video call")
    } else {
        gettext("Incoming call")
    }
}

fn message_basic_group_chat_create(title: &str, sender: &model::MessageSender) -> String {
    let sender = message_sender(sender, true);
    gettext_f(
        "{sender} created the group \"{title}\"",
        &[("sender", &sender), ("title", title)],
    )
}

fn message_supergroup_chat_create(
    title: &str,
    chat: &model::Chat,
    sender: &model::MessageSender,
) -> String {
    match chat.chat_type() {
        model::ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
            gettext("Channel created")
        }
        _ => {
            let sender = message_sender(sender, true);
            gettext_f(
                "{sender} created the group \"{title}\"",
                &[("sender", &sender), ("title", title)],
            )
        }
    }
}

fn message_chat_change_title(
    title: &str,
    chat: &model::Chat,
    sender: &model::MessageSender,
) -> String {
    match chat.chat_type() {
        model::ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
            gettext_f("Channel renamed to \"{title}\"", &[("title", title)])
        }
        _ => {
            let sender = message_sender(sender, true);
            gettext_f(
                "{sender} changed the group name to \"{title}\"",
                &[("sender", &sender), ("title", title)],
            )
        }
    }
}

fn message_chat_change_photo(chat: &model::Chat, sender: &model::MessageSender) -> String {
    match chat.chat_type() {
        model::ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
            gettext("Channel photo changed")
        }
        _ => {
            let sender = message_sender(sender, true);
            gettext_f("{sender} changed the group photo", &[("sender", &sender)])
        }
    }
}

fn message_chat_delete_photo(chat: &model::Chat, sender: &model::MessageSender) -> String {
    match chat.chat_type() {
        model::ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
            gettext("Channel photo removed")
        }
        _ => {
            let sender = message_sender(sender, true);
            gettext_f("{sender} removed the group photo", &[("sender", &sender)])
        }
    }
}

fn message_chat_add_members(sender: &model::MessageSender, added_users: &[model::User]) -> String {
    let sender_string = message_sender(sender, true);
    if sender.as_user().map(model::User::id) == added_users.first().map(model::User::id) {
        gettext_f("{sender} joined the group", &[("sender", &sender_string)])
    } else if added_users.len() == 2 {
        let first_user = user_display_name(added_users.first().unwrap(), true);
        let second_user = user_display_name(added_users.last().unwrap(), true);
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
            .map(|u| user_display_name(u, true))
            .collect::<Vec<_>>()
            .join(", ");
        gettext_f(
            "{sender} added {users}",
            &[("sender", &sender_string), ("users", &users)],
        )
    }
}

fn message_chat_join_by_link(sender: &model::MessageSender) -> String {
    let sender = message_sender(sender, true);
    gettext_f(
        "{sender} joined the group via invite link",
        &[("sender", &sender)],
    )
}

fn message_chat_join_by_request(sender: &model::MessageSender) -> String {
    let sender = message_sender(sender, true);
    gettext_f("{sender} joined the group", &[("sender", &sender)])
}

fn message_chat_delete_member(deleted_user: &model::User, sender: &model::MessageSender) -> String {
    let sender_string = message_sender(sender, true);
    match sender {
        model::MessageSender::User(user) if user.id() == deleted_user.id() => {
            gettext_f("{sender} left the group", &[("sender", &sender_string)])
        }
        _ => {
            let deleted_user_name = user_display_name(deleted_user, true);
            gettext_f(
                "{sender} removed {user}",
                &[("sender", &sender_string), ("user", &deleted_user_name)],
            )
        }
    }
}

fn message_game_score(
    game: &tdlib::types::MessageGameScore,
    chat: &model::Chat,
    sender: &model::MessageSender,
) -> String {
    let sender_string = message_sender(sender, true);
    let game_title = match chat.message(game.game_message_id) {
        Some(message) => match message.content().0 {
            tdlib::enums::MessageContent::MessageGame(tdlib::types::MessageGame { game }) => {
                Some(game.title)
            }
            _ => unreachable!(),
        },
        None => None,
    };

    if let Some(game_title) = game_title {
        gettext_f(
            "{sender} scored {points} in {game}",
            &[
                ("sender", &sender_string),
                ("points", &game.score.to_string()),
                ("game", &game_title),
            ],
        )
    } else {
        gettext_f(
            "{sender} scored {points}",
            &[
                ("sender", &sender_string),
                ("points", &game.score.to_string()),
            ],
        )
    }
}

fn message_pin_message(id: MessageId, chat: &model::Chat, sender: &model::MessageSender) -> String {
    use tdlib::enums::MessageContent::*;

    // TODO: Add a way to retrieve the message and update the string
    // in case we don't have the message stored locally.
    let string = match chat.message(id) {
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

    let sender = message_sender(sender, true);
    utils::freplace(string, &[("sender", &sender)])
}

fn message_contact_registered(sender: &model::MessageSender) -> String {
    let sender = message_sender(sender, true);
    gettext_f("{sender} joined Telegram", &[("sender", &sender)])
}
