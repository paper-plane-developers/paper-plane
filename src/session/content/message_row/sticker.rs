use adw::prelude::*;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};
use image::io::Reader as ImageReader;
use image::ImageFormat;
use std::io::Cursor;
use tdlib::enums::{MessageContent, StickerFormat};
use tdlib::types::File;

use crate::session::content::message_row::{MessageBase, MessageBaseImpl, MessageIndicators};
use crate::tdlib::Message;
use crate::utils::spawn;

use super::base::MessageBaseExt;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-sticker.ui")]
    pub(crate) struct MessageSticker {
        pub(super) format: OnceCell<StickerFormat>,
        pub(super) aspect_ratio: Cell<f64>,
        pub(super) message: RefCell<Option<Message>>,
        #[template_child]
        pub(super) overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub(super) bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) indicators: TemplateChild<MessageIndicators>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageSticker {
        const NAME: &'static str = "ContentMessageSticker";
        type Type = super::MessageSticker;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("messagesticker");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageSticker {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<Message>("message")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "message" => obj.set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => self.message.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageSticker {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            const SIZE: i32 = 208;
            let aspect_ratio = self.aspect_ratio.get();
            let min_size = self.overlay.measure(orientation, for_size).0;
            let size = if let gtk::Orientation::Horizontal = orientation {
                if aspect_ratio >= 1.0 {
                    SIZE
                } else {
                    (SIZE as f64 * aspect_ratio) as i32
                }
            } else if aspect_ratio >= 1.0 {
                (SIZE as f64 / aspect_ratio) as i32
            } else {
                SIZE
            }
            .max(min_size);

            (size, size, -1, -1)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.overlay.allocate(width, height, baseline, None);
        }
    }
    impl MessageBaseImpl for MessageSticker {}
}

glib::wrapper! {
    pub(crate) struct MessageSticker(ObjectSubclass<imp::MessageSticker>)
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessageSticker {
    type Message = Message;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        imp.indicators.set_message(message.clone().upcast());

        if let MessageContent::MessageSticker(data) = message.content().0 {
            imp.format.set(data.sticker.format).unwrap();

            imp.aspect_ratio
                .set(data.sticker.width as f64 / data.sticker.height as f64);

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
        imp.message.replace(Some(message));

        self.notify("message");
    }
}

impl MessageSticker {
    fn load_sticker(&self, path: &str) {
        let path = path.to_owned();
        spawn(clone!(@weak self as obj => async move {
            let widget: gtk::Widget = match obj.imp().format.get().unwrap() {
            StickerFormat::Tgs => {
                let animation = rlt::Animation::from_filename(&path);
                animation.set_loop(true);
                animation.use_cache(true);
                animation.play();
                animation.upcast()
            }
            StickerFormat::Webp => {
                let file = gio::File::for_path(&path);
                match file.load_bytes_future().await {
                    Ok((bytes, _)) => {
                        let flat_samples =
                            ImageReader::with_format(Cursor::new(bytes), ImageFormat::WebP)
                                .decode()
                                .unwrap()
                                .into_rgba8()
                                .into_flat_samples();

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

                        let picture = gtk::Picture::new();

                        picture.set_paintable(Some(&texture));

                        picture.upcast()
                    }
                    Err(e) => {
                        log::warn!("Failed to load a sticker: {}", e);
                        return;
                    }
                }
            }
            _ => unimplemented!(),
        };

        obj.imp().bin.set_child(Some(&widget));

        }));
    }
}
