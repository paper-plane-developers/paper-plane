use crate::i18n::gettext_f;
use crate::strings;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::closure;
use gtk::prelude::GObjectPropertyExpressionExt;

use crate::tdlib::{
    BasicGroup, BoxedChatMemberStatus, BoxedChatPermissions, BoxedUserStatus, BoxedUserType, Chat,
    ChatActionList, ChatType, Supergroup, User,
};
use tdlib::enums::{ChatMemberStatus, UserType};

/// Creates an expression that produces the display name of a chat. This will either produce the
/// title of the chat or the translated "Saved Messages" string in the case of the own chat.
pub(crate) fn chat_display_name(chat_expression: &gtk::Expression) -> gtk::Expression {
    let title_expression = chat_expression.chain_property::<Chat>("title");
    let is_deleted_expression = is_deleted_expression(chat_expression);
    gtk::ClosureExpression::with_callback(
        [chat_expression, &title_expression, &is_deleted_expression],
        |args| {
            let chat = args[1].get::<Chat>().unwrap();
            let title = args[2].get::<String>().unwrap();
            let is_deleted = args[3].get::<bool>().unwrap();
            if chat.is_own_chat() {
                gettext("Saved Messages")
            } else if is_deleted {
                gettext("Deleted Account")
            } else {
                title
            }
        },
    )
    .upcast()
}

/// Creates an expression that produces the full name of an user, binding both the
/// first-name and last-name property together.
pub(crate) fn user_display_name(user_expression: &gtk::Expression) -> gtk::Expression {
    let first_name_expression = user_expression.chain_property::<User>("first-name");
    let last_name_expression = user_expression.chain_property::<User>("last-name");
    let type_expression = user_expression.chain_property::<User>("type");
    gtk::ClosureExpression::with_callback(
        &[first_name_expression, last_name_expression, type_expression],
        |args| {
            let first_name = args[1].get::<String>().unwrap();
            let last_name = args[2].get::<String>().unwrap();
            let user_type = args[3].get::<BoxedUserType>().unwrap().0;
            if let UserType::Deleted = user_type {
                gettext("Deleted Account")
            } else {
                first_name + " " + &last_name
            }
        },
    )
    .upcast()
}

pub(crate) fn is_deleted_expression(chat_expression: &gtk::Expression) -> gtk::Expression {
    gtk::ClosureExpression::with_callback([chat_expression], |args| {
        let chat = args[1].get::<Chat>().unwrap();
        matches!(chat.type_(), ChatType::Private(user) if user.type_().0 == UserType::Deleted)
    })
    .upcast()
}

pub(crate) fn restriction_expression(chat: &Chat) -> gtk::Expression {
    match chat.type_() {
        ChatType::Supergroup(data) if !data.is_channel() => {
            restriction_label_expression::<Supergroup, _>(data)
        }
        ChatType::BasicGroup(data) => restriction_label_expression::<BasicGroup, _>(data),
        _ => gtk::ConstantExpression::new("").upcast(),
    }
}

fn restriction_label_expression<T: glib::StaticType, V: glib::ToValue>(
    value: &V,
) -> gtk::Expression {
    let member_status_expression = gtk::PropertyExpression::new(
        T::static_type(),
        Some(gtk::ConstantExpression::new(value)),
        "status",
    );
    let permissions_expression = Chat::this_expression("permissions");

    gtk::ClosureExpression::new::<String>(
        &[member_status_expression, permissions_expression],
        closure!(|_: glib::Object, status: BoxedChatMemberStatus, chat_permissions: BoxedChatPermissions| {
            if chat_permissions.0.can_send_basic_messages {
                match status.0 {
                    ChatMemberStatus::Restricted(status) if !status.permissions.can_send_basic_messages => {
                        if status.restricted_until_date == 0 {
                            gettext("The admins of this group have restricted you from writing here")
                        } else {
                            let date =
                            glib::DateTime::from_unix_utc(status.restricted_until_date.into()).unwrap();

                            gettext!(
                                "The admins of this group have restricted you from writing here until {}",
                                date.format(&if glib::DateTime::now_local()
                                    .unwrap()
                                    .difference(&date)
                                    .as_days()
                                    == 0
                                {
                                    gettext("%l:%M %p")
                                } else {
                                    // Translators: This is a date and time representation, without seconds.
                                    // Here you may want to change to a 24-hours representation and change order, based on your locale.
                                    // You can use this site to learn more: https://www.strfti.me/
                                    gettext("%B %e, %Y %l:%M %p")
                                })
                                .unwrap()
                                .to_string()
                            )
                        }
                    }
                    _ => String::new(),
                }
            } else if !matches!(status.0, ChatMemberStatus::Creator(_) | ChatMemberStatus::Administrator(_)) {
                gettext("Writing messages isn't allowed in this group.")
            } else {
                String::new()
            }
        }),
    )
    .upcast()
}

pub(crate) fn subtitle_expression(chat: &Chat) -> gtk::Expression {
    let actions_expression = gtk::ConstantExpression::new(chat)
        .chain_property::<Chat>("actions")
        .upcast();
    let online_count_expression =
        gtk::ConstantExpression::new(chat).chain_property::<Chat>("online-member-count");

    let status_expression = if !chat.is_own_chat() {
        match chat.type_() {
            ChatType::Private(user) => {
                if let UserType::Bot(_) = user.type_().0 {
                    gtk::ConstantExpression::new(gettext("bot")).upcast()
                } else {
                    gtk::ConstantExpression::new(user)
                        .chain_property::<User>("status")
                        .chain_closure::<String>(closure!(
                            |_: glib::Object, status: BoxedUserStatus| {
                                strings::user_status(&status.0)
                            }
                        ))
                        .upcast()
                }
            }
            ChatType::Secret(data) => gtk::ConstantExpression::new(data.user())
                .chain_property::<User>("status")
                .chain_closure::<String>(closure!(|_: glib::Object, status: BoxedUserStatus| {
                    strings::user_status(&status.0)
                }))
                .upcast(),
            ChatType::Supergroup(data) => {
                if data.is_channel() {
                    gtk::ConstantExpression::new(data)
                        .chain_property::<Supergroup>("member-count")
                        .chain_closure::<String>(closure!(|_: glib::Object, sub_count: i32| {
                            match sub_count {
                                0 => gettext("channel"),
                                1 => gettext("1 subscriber"),
                                _ => gettext_f(
                                    "{count} subscribers",
                                    &[("count", &sub_count.to_string())],
                                ),
                            }
                        }))
                        .upcast()
                } else {
                    let member_count_expression = gtk::ConstantExpression::new(data)
                        .chain_property::<Supergroup>("member-count");
                    gtk::ClosureExpression::with_callback(
                        &[member_count_expression, online_count_expression],
                        |args| {
                            strings::group_subtitle(
                                args[1].get::<i32>().unwrap(),
                                args[2].get::<i32>().unwrap(),
                            )
                        },
                    )
                    .upcast()
                }
            }
            ChatType::BasicGroup(data) => {
                let member_count_expression =
                    gtk::ConstantExpression::new(data).chain_property::<BasicGroup>("member-count");
                gtk::ClosureExpression::with_callback(
                    &[member_count_expression, online_count_expression],
                    |args| {
                        strings::group_subtitle(
                            args[1].get::<i32>().unwrap(),
                            args[2].get::<i32>().unwrap(),
                        )
                    },
                )
                .upcast()
            }
        }
    } else {
        gtk::ConstantExpression::new(String::new()).upcast()
    };

    gtk::ClosureExpression::with_callback(&[actions_expression, status_expression], |args| {
        let actions = args[1].get::<ChatActionList>().unwrap();
        let status = args[2].get::<String>().unwrap();
        if let Some(action) = actions.last() {
            strings::chat_action(&action)
        } else {
            status
        }
    })
    .upcast()
}
