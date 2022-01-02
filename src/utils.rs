use gettextrs::gettext;
use gtk::glib;
use once_cell::sync::Lazy;
use regex::Regex;
use std::future::Future;
use tdgrand::enums::TextEntityType;
use tdgrand::types::FormattedText;

use crate::RUNTIME;

pub static PROTOCOL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+://").unwrap());

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

pub fn dim(text: &str) -> String {
    // The alpha value should be kept in sync with Adwaita's dim-label alpha value
    format!("<span alpha=\"55%\">{}</span>", text)
}

pub fn dim_and_escape(text: &str) -> String {
    dim(&escape(text))
}

pub fn linkify(text: &str) -> String {
    if !PROTOCOL_RE.is_match(text) {
        format!("http://{}", text)
    } else {
        text.to_string()
    }
}

pub fn convert_to_markup(text: String, entity: &TextEntityType) -> String {
    match entity {
        TextEntityType::Url => format!("<a href='{}'>{}</a>", linkify(&text), text),
        TextEntityType::EmailAddress => format!("<a href='mailto:{0}'>{0}</a>", text),
        TextEntityType::PhoneNumber => format!("<a href='tel:{0}'>{0}</a>", text),
        TextEntityType::Bold => format!("<b>{}</b>", text),
        TextEntityType::Italic => format!("<i>{}</i>", text),
        TextEntityType::Underline => format!("<u>{}</u>", text),
        TextEntityType::Strikethrough => format!("<s>{}</s>", text),
        TextEntityType::Code | TextEntityType::Pre | TextEntityType::PreCode(_) => {
            format!("<tt>{}</tt>", text)
        }
        TextEntityType::TextUrl(data) => format!("<a href='{}'>{}</a>", escape(&data.url), text),
        _ => text,
    }
}

pub fn parse_formatted_text(formatted_text: FormattedText) -> String {
    let mut entities = formatted_text.entities.iter();
    let mut entity = entities.next();
    let mut output = String::new();
    let mut buffer = String::new();
    let mut is_inside_entity = false;

    // This is the offset in utf16 code units of the text to parse. We need this variable
    // because tdlib stores the offset and length parameters as utf16 code units instead
    // of regular code points.
    let mut code_units_offset = 0;

    for c in formatted_text.text.chars() {
        if !is_inside_entity
            && entity.is_some()
            && code_units_offset >= entity.unwrap().offset as usize
        {
            is_inside_entity = true;

            if !buffer.is_empty() {
                output.push_str(&escape(&buffer));
                buffer = String::new();
            }
        }

        buffer.push(c);
        code_units_offset += c.len_utf16();

        if let Some(entity_) = entity {
            if code_units_offset >= (entity_.offset + entity_.length) as usize {
                buffer = escape(&buffer);

                entity = loop {
                    let entity = entities.next();

                    // Handle eventual nested entities
                    match entity {
                        Some(entity) => {
                            if entity.offset == entity_.offset {
                                buffer = convert_to_markup(buffer, &entity.r#type);
                            } else {
                                break Some(entity);
                            }
                        }
                        None => break None,
                    }
                };

                output.push_str(&convert_to_markup(buffer, &entity_.r#type));
                buffer = String::new();
                is_inside_entity = false;
            }
        }
    }

    // Add the eventual leftovers from the buffer to the output
    if !buffer.is_empty() {
        output.push_str(&escape(&buffer));
    }

    output
}

pub fn human_friendly_duration(mut seconds: i32) -> String {
    let hours = seconds / (60 * 60);
    if hours > 0 {
        seconds %= 60 * 60;
        let minutes = seconds / 60;
        if minutes > 0 {
            seconds %= 60;
            gettext!("{} h {} min {} s", hours, minutes, seconds)
        } else {
            gettext!("{} h {} s", hours, seconds)
        }
    } else {
        let minutes = seconds / 60;
        if minutes > 0 {
            seconds %= 60;
            if seconds > 0 {
                gettext!("{} min {} s", minutes, seconds)
            } else {
                gettext!("{} min", minutes)
            }
        } else {
            gettext!("{} s", seconds)
        }
    }
}

// Function from https://gitlab.gnome.org/GNOME/fractal/-/blob/fractal-next/src/utils.rs
pub fn do_async<
    R: Send + 'static,
    F1: Future<Output = R> + Send + 'static,
    F2: Future<Output = ()> + 'static,
    FN: FnOnce(R) -> F2 + 'static,
>(
    priority: glib::source::Priority,
    tokio_fut: F1,
    glib_closure: FN,
) {
    let (sender, receiver) = tokio::sync::oneshot::channel();

    glib::MainContext::default().spawn_local_with_priority(priority, async move {
        glib_closure(receiver.await.unwrap()).await
    });

    RUNTIME.spawn(async move { sender.send(tokio_fut.await) });
}

/// Spawn a future on the default `MainContext`
///
/// This was taken from `gtk-macros` and `fractal`
#[macro_export]
macro_rules! spawn {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local($future);
    };
    ($priority:expr, $future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local_with_priority($priority, $future);
    };
}
