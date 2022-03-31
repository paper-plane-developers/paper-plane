use crate::session::User;

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
