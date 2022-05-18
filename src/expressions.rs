use gettextrs::gettext;
use gtk::glib;
use gtk::glib::closure;
use gtk::prelude::GObjectPropertyExpressionExt;

use crate::tdlib::{
    BasicGroup, BoxedChatMemberStatus, BoxedChatPermissions, Chat, ChatType, Supergroup, User,
};
use tdlib::enums::ChatMemberStatus;

/// Creates an expression that produces the display name of a chat. This will either produce the
/// title of the chat or the translated "Saved Messages" string in the case of the own chat.
pub(crate) fn chat_display_name(chat_expression: &gtk::Expression) -> gtk::Expression {
    let title_expression = chat_expression.chain_property::<Chat>("title");
    gtk::ClosureExpression::with_callback(&[chat_expression, &title_expression], |args| {
        let chat = args[1].get::<Chat>().unwrap();
        let title = args[2].get::<String>().unwrap();
        if chat.is_own_chat() {
            gettext("Saved Messages")
        } else {
            title
        }
    })
    .upcast()
}

/// Creates an expression that produces the full name of an user, binding both the
/// first-name and last-name property together.
pub(crate) fn user_full_name(user_expression: &gtk::Expression) -> gtk::Expression {
    let first_name_expression = user_expression.chain_property::<User>("first-name");
    let last_name_expression = user_expression.chain_property::<User>("last-name");
    gtk::ClosureExpression::with_callback(&[first_name_expression, last_name_expression], |args| {
        let first_name = args[1].get::<String>().unwrap();
        let last_name = args[2].get::<String>().unwrap();
        first_name + " " + &last_name
    })
    .upcast()
}

pub(crate) fn restriction_expression(chat: &Chat) -> gtk::Expression {
    match chat.type_() {
        ChatType::Supergroup(data) if !data.is_channel() => {
            restriction_label_expression::<Supergroup, _>(data)
        }
        ChatType::BasicGroup(data) => restriction_label_expression::<BasicGroup, _>(data),
        _ => gtk::ConstantExpression::new(&"").upcast(),
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

    gtk::ClosureExpression::new::<String, _, _>(
        &[member_status_expression, permissions_expression],
        closure!(|_: glib::Object, status: BoxedChatMemberStatus, chat_permissions: BoxedChatPermissions| {
            if chat_permissions.0.can_send_messages {
                match status.0 {
                    ChatMemberStatus::Restricted(status) if !status.permissions.can_send_messages => {
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
                                    // Translators: This is a hours: minutes time format used in restriction label
                                    gettext("%l:%M %p")
                                } else {
                                    // Translators: This is a full date format used in restriction label
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
