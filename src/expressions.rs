use gettextrs::gettext;

use crate::session::{Chat, User};

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
