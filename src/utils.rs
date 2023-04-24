use std::future::Future;
use std::path::PathBuf;

use gettextrs::gettext;
use gtk::gdk;
use gtk::glib;
use gtk::graphene;
use image::io::Reader as ImageReader;
use locale_config::Locale;
use once_cell::sync::Lazy;
use regex::Regex;
use tdlib::enums::TextEntityType;
use tdlib::functions;
use tdlib::types;
use tdlib::types::FormattedText;
use thiserror::Error;

use crate::config;
use crate::session_manager::DatabaseInfo;
use crate::APPLICATION_OPTS;
use crate::TEMP_DIR;

static PROTOCOL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+://").unwrap());

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
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

pub(crate) fn linkify(text: &str) -> String {
    if !PROTOCOL_RE.is_match(text) {
        format!("http://{text}")
    } else {
        text.to_string()
    }
}

pub(crate) fn convert_to_markup(text: String, entity: &TextEntityType) -> String {
    match entity {
        TextEntityType::Url => format!("<a href='{}'>{}</a>", linkify(&text), text),
        TextEntityType::EmailAddress => format!("<a href='mailto:{text}'>{text}</a>"),
        TextEntityType::PhoneNumber => format!("<a href='tel:{text}'>{text}</a>"),
        TextEntityType::Bold => format!("<b>{text}</b>"),
        TextEntityType::Italic => format!("<i>{text}</i>"),
        TextEntityType::Underline => format!("<u>{text}</u>"),
        TextEntityType::Strikethrough => format!("<s>{text}</s>"),
        TextEntityType::Code | TextEntityType::Pre | TextEntityType::PreCode(_) => {
            format!("<tt>{text}</tt>")
        }
        TextEntityType::TextUrl(data) => format!("<a href='{}'>{}</a>", escape(&data.url), text),
        TextEntityType::CustomEmoji(_) => "<widget>\u{200B}".to_string(),
        _ => text,
    }
}

pub(crate) fn custom_emoji_ids(entities: &[tdlib::types::TextEntity]) -> Vec<i64> {
    entities
        .iter()
        .flat_map(|entity| match &entity.r#type {
            TextEntityType::CustomEmoji(emoji) => Some(emoji.custom_emoji_id),
            _ => None,
        })
        .collect()
}

pub(crate) async fn custom_emojis(session: crate::Session, ids: Vec<i64>) -> Vec<gtk::Widget> {
    let client_id = session.client_id();

    if let Ok(stickers) = tdlib::functions::get_custom_emoji_stickers(ids.clone(), client_id).await
    {
        let tdlib::enums::Stickers::Stickers(stickers) = stickers;
        use crate::components::Sticker;
        use gtk::prelude::*;

        let mut sticker_map = std::collections::BTreeMap::new();
        for sticker in stickers.stickers {
            sticker_map.insert(sticker.id, sticker);
        }

        let widgets = ids
            .iter()
            .map(|id| {
                let sticker = sticker_map[id].clone();
                let widget = Sticker::new();
                widget.set_longer_side_size(20);
                widget.update_sticker(sticker, true, session.clone());
                widget.upcast()
            })
            .collect();

        widgets
    } else {
        log::error!("Failed to get custom emojis");
        vec![]
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

/// Returns the Paper Plane data directory (e.g. /home/bob/.local/share/paper-plane).
pub(crate) fn data_dir() -> &'static PathBuf {
    &APPLICATION_OPTS.get().unwrap().data_dir
}

/// Returns the Paper Plane temp directory (e.g. /tmp/paper-plane2-0).
pub(crate) fn temp_dir() -> Option<&'static PathBuf> {
    TEMP_DIR.get()
}

pub(crate) async fn send_tdlib_parameters(
    client_id: i32,
    database_info: &DatabaseInfo,
) -> Result<(), types::Error> {
    let system_language_code = {
        let locale = Locale::current().to_string();
        if !locale.is_empty() {
            locale
        } else {
            "en_US".to_string()
        }
    };

    let database_directory = data_dir()
        .join(&database_info.directory_base_name)
        .to_str()
        .expect("Data directory path is not a valid unicode string")
        .into();

    functions::set_tdlib_parameters(
        database_info.use_test_dc,
        database_directory,
        String::new(),
        String::new(),
        true,
        true,
        true,
        true,
        config::TG_API_ID,
        config::TG_API_HASH.into(),
        system_language_code,
        "Desktop".into(),
        String::new(),
        config::VERSION.into(),
        true,
        false,
        client_id,
    )
    .await
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

#[derive(Error, Debug)]
pub(crate) enum DecodeError {
    #[error("I/O error: {0:?}")]
    IoError(std::io::Error),
    #[error("Image decoding error: {0:?}")]
    ImageError(image::error::ImageError),
    #[error("Decoding for this image format is not currently implemented")]
    Unimplemented,
}

pub(crate) fn decode_image_from_path(path: &str) -> Result<gdk::MemoryTexture, DecodeError> {
    use image::DynamicImage::*;

    let dynamic_image = ImageReader::open(path)
        .map_err(DecodeError::IoError)?
        .decode()
        .map_err(DecodeError::ImageError)?;
    let (memory_format, layout, data) = match dynamic_image {
        ImageLuma8(_) => {
            let buffer = dynamic_image.to_rgb8();
            (
                gdk::MemoryFormat::R8g8b8,
                buffer.sample_layout(),
                buffer.into_raw(),
            )
        }
        ImageLumaA8(_) => {
            let buffer = dynamic_image.to_rgba8();
            (
                gdk::MemoryFormat::R8g8b8a8,
                buffer.sample_layout(),
                buffer.into_raw(),
            )
        }
        ImageRgb8(buffer) => (
            gdk::MemoryFormat::R8g8b8,
            buffer.sample_layout(),
            buffer.into_raw(),
        ),
        ImageRgba8(buffer) => (
            gdk::MemoryFormat::R8g8b8a8,
            buffer.sample_layout(),
            buffer.into_raw(),
        ),
        _ => return Err(DecodeError::Unimplemented),
    };

    let bytes = glib::Bytes::from_owned(data);
    let texture = gdk::MemoryTexture::new(
        layout.width as i32,
        layout.height as i32,
        memory_format,
        &bytes,
        layout.height_stride,
    );

    Ok(texture)
}

pub fn color_matrix_from_color(color: gdk::RGBA) -> (graphene::Matrix, graphene::Vec4) {
    let color_offset = graphene::Vec4::new(color.red(), color.green(), color.blue(), 0.0);
    let mut matrix = [0.0; 16];
    matrix[15] = color.alpha();
    let color_matrix = graphene::Matrix::from_float(matrix);

    (color_matrix, color_offset)
}
