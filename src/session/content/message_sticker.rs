use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums::MessageContent, types::File};

use crate::session::chat::Message;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-sticker.ui")]
    pub struct MessageSticker {
        #[template_child]
        pub sticker_picture: TemplateChild<gtk::Picture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageSticker {
        const NAME: &'static str = "ContentMessageSticker";
        type Type = super::MessageSticker;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageSticker {}
    impl WidgetImpl for MessageSticker {}
    impl BinImpl for MessageSticker {}
}

glib::wrapper! {
    pub struct MessageSticker(ObjectSubclass<imp::MessageSticker>)
        @extends gtk::Widget, adw::Bin;
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
                .sticker_picture
                .set_height_request((data.sticker.height as f32 / 2.3) as i32);

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
        self_.sticker_picture.set_paintable(Some(&media));
    }
}
