use gettextrs::gettext;

use crate::utils::freplace;

/// Like `gettext`, but replaces named variables with the given dictionary.
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
// Function taken from Fractal: https://gitlab.gnome.org/GNOME/fractal/-/blob/main/src/i18n.rs
pub(crate) fn gettext_f(msgid: &str, args: &[(&str, &str)]) -> String {
    let s = gettext(msgid);
    freplace(s, args)
}
