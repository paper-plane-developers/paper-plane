use gettextrs::{gettext, ngettext};

use crate::utils::freplace;

// Module taken from Fractal: https://gitlab.gnome.org/GNOME/fractal/-/blob/main/src/i18n.rs

/// Like `gettext`, but replaces named variables with the given dictionary.
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
pub(crate) fn gettext_f(msgid: &str, args: &[(&str, &str)]) -> String {
    let s = gettext(msgid);
    freplace(s, args)
}

/// Like `ngettext`, but replaces named variables with the given dictionary.
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
pub fn ngettext_f(msgid: &str, msgid_plural: &str, n: u32, args: &[(&str, &str)]) -> String {
    let s = ngettext(msgid, msgid_plural, n);
    freplace(s, args)
}
