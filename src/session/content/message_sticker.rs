use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums::MessageContent, types::File};

use crate::session::chat::Message;
use crate::session::content::MessageIndicators;

mod imp {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-sticker.ui")]
    pub struct MessageSticker {
        pub width: Cell<i32>,
        pub height: Cell<i32>,
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub indicators: TemplateChild<MessageIndicators>,
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
        fn dispose(&self, _obj: &Self::Type) {
            self.overlay.unparent();
        }
    }

    impl WidgetImpl for MessageSticker {
        fn measure(
            &self,
            _widget: &Self::Type,
            orientation: gtk::Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let size = if let gtk::Orientation::Horizontal = orientation {
                self.width.get()
            } else {
                self.height.get()
            };

            (size, size, -1, -1)
        }

        fn size_allocate(&self, _widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.overlay.allocate(width, height, baseline, None);
        }
    }
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

            self_.indicators.set_message(message);

            // Scale the sticker to fit it in a small container, but keeping its
            // original aspect ratio
            let max_size = 200;
            if data.sticker.width > data.sticker.height {
                self_.width.set(max_size);
                self_
                    .height
                    .set(data.sticker.height * max_size / data.sticker.width);
            } else {
                self_.height.set(max_size);
                self_
                    .width
                    .set(data.sticker.width * max_size / data.sticker.height);
            }

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
        let media = gtk::MediaFile::for_filename(path);
        self_.picture.set_paintable(Some(&media));
    }
}
