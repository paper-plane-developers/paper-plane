use glib::clone;
use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums::MessageContent, types::File};

use crate::session::chat::Message;
use crate::session::content::message_row::StickerPaintable;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-sticker.ui")]
    pub struct MessageSticker {
        pub paintable: StickerPaintable,
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageSticker {
        const NAME: &'static str = "ContentMessageSticker";
        type Type = super::MessageSticker;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageSticker {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.picture.set_paintable(Some(&self.paintable));
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.picture.unparent();
        }
    }

    impl WidgetImpl for MessageSticker {}
}

glib::wrapper! {
    pub struct MessageSticker(ObjectSubclass<imp::MessageSticker>)
        @extends gtk::Widget;
}

impl Default for MessageSticker {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageSticker {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageSticker")
    }

    pub fn set_message(&self, message: &Message) {
        if let MessageContent::MessageSticker(data) = message.content().0 {
            let self_ = imp::MessageSticker::from_instance(self);
            self_
                .paintable
                .set_aspect_ratio(data.sticker.width as f64 / data.sticker.height as f64);

            if data.sticker.sticker.local.is_downloading_completed {
                self.load_sticker(&data.sticker.sticker.local.path);
            } else {
                let (sender, receiver) =
                    glib::MainContext::sync_channel::<File>(Default::default(), 5);

                receiver.attach(
                    None,
                    clone!(@weak self as obj => @default-return glib::Continue(false), move |file| {
                        if file.local.is_downloading_completed {
                            obj.load_sticker(&file.local.path);
                        }

                        glib::Continue(true)
                    }),
                );

                message
                    .chat()
                    .session()
                    .download_file(data.sticker.sticker.id, sender);
            }
        }
    }

    fn load_sticker(&self, path: &str) {
        let self_ = imp::MessageSticker::from_instance(self);
        let file = gio::File::for_path(path);
        let future = clone!(@weak self_.paintable as paintable => async move {
            match file.load_bytes_async_future().await {
                Ok((bytes, _)) => {
                    let image = webp::Decoder::new(&bytes)
                        .decode()
                        .unwrap()
                        .to_image()
                        .into_rgba8();

                    let flat_samples = image.into_flat_samples();

                    let (stride, width, height) = flat_samples.extents();
                    let gtk_stride = stride * width;

                    let bytes = glib::Bytes::from_owned(flat_samples.samples);
                    let texture = gdk::MemoryTexture::new(
                        width as i32,
                        height as i32,
                        gdk::MemoryFormat::R8g8b8a8,
                        &bytes,
                        gtk_stride,
                    );
                    paintable.set_texture(Some(texture.upcast()));
                }
                Err(e) => {
                    log::warn!("Failed to load a sticker: {}", e);
                }
            }
        });

        glib::MainContext::default().spawn_local(future);
    }
}
