use gettextrs::gettext;
use gtk::glib;
use locale_config::Locale;
use once_cell::sync::Lazy;
use regex::Regex;
use std::future::Future;
use std::path::PathBuf;
use tdlib::enums::TextEntityType;
use tdlib::types::{self, FormattedText};
use tdlib::{enums, functions};

use crate::session_manager::DatabaseInfo;
use crate::{config, APPLICATION_OPTS, TEMP_DIR};

static PROTOCOL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+://").unwrap());

pub(crate) const MESSAGE_TRUNCATED_LENGTH: usize = 21;

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

/// Replace variables in the given string with the given dictionary.
///
/// The expected format to replace is `{name}`, where `name` is the first string
/// in the dictionary entry tuple.
// Function taken from Fractal: https://gitlab.gnome.org/GNOME/fractal/-/blob/main/src/utils.rs
pub(crate) fn freplace(s: String, args: &[(&str, &str)]) -> String {
    let mut s = s;
    for (k, v) in args {
        s = s.replace(&format!("{{{}}}", k), v);
    }
    s
}

pub(crate) fn dim(text: &str) -> String {
    // The alpha value should be kept in sync with Adwaita's dim-label alpha value
    format!("<span alpha=\"55%\">{}</span>", text)
}

pub(crate) fn dim_and_escape(text: &str) -> String {
    dim(&escape(text))
}

pub(crate) fn linkify(text: &str) -> String {
    if !PROTOCOL_RE.is_match(text) {
        format!("http://{}", text)
    } else {
        text.to_string()
    }
}

pub(crate) fn convert_to_markup(text: String, entity: &TextEntityType) -> String {
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

pub(crate) fn parse_formatted_text(formatted_text: FormattedText) -> String {
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

pub(crate) fn human_friendly_duration(mut seconds: i32) -> String {
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

/// Returns the Telegrand data directory (e.g. /home/bob/.local/share/telegrand).
pub(crate) fn data_dir() -> &'static PathBuf {
    &APPLICATION_OPTS.get().unwrap().data_dir
}

/// Returns the Telegrand temp directory (e.g. /tmp/telegrand2-0).
pub(crate) fn temp_dir() -> Option<&'static PathBuf> {
    TEMP_DIR.get()
}

pub(crate) async fn send_tdlib_parameters(
    client_id: i32,
    database_info: &DatabaseInfo,
) -> Result<enums::Ok, types::Error> {
    let system_language_code = {
        let locale = Locale::current().to_string();
        if !locale.is_empty() {
            locale
        } else {
            "en_US".to_string()
        }
    };
    let parameters = types::TdlibParameters {
        use_test_dc: database_info.use_test_dc,
        database_directory: data_dir()
            .join(&database_info.directory_base_name)
            .to_str()
            .expect("Data directory path is not a valid unicode string")
            .to_owned(),
        use_message_database: true,
        use_secret_chats: true,
        api_id: config::TG_API_ID,
        api_hash: config::TG_API_HASH.to_string(),
        system_language_code,
        device_model: "Desktop".to_string(),
        application_version: config::VERSION.to_string(),
        enable_storage_optimizer: true,
        ..types::TdlibParameters::default()
    };

    functions::set_tdlib_parameters(parameters, client_id).await
}

pub(crate) async fn log_out(client_id: i32) {
    if let Err(e) = functions::log_out(client_id).await {
        log::error!("Could not logout client with id={}: {:?}", client_id, e);
    }
}

/// Spawn a future on the default `MainContext`
pub(crate) fn spawn<F: Future<Output = ()> + 'static>(fut: F) {
    let ctx = glib::MainContext::default();
    ctx.spawn_local(fut);
}

/// Run a future on the default `MainContext` and block until finished
pub(crate) fn block_on<F: Future>(fut: F) -> F::Output {
    let ctx = glib::MainContext::default();
    ctx.block_on(fut)
}
