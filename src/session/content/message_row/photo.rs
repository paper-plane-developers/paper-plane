use glib::{clone, closure};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};
use tdlib::enums::MessageContent;
use tdlib::types::File;

use crate::session::chat::{BoxedMessageContent, Message};
use crate::session::content::message_row::Media;
use crate::session::content::{MessageRow, MessageRowExt};
use crate::utils::parse_formatted_text;
use crate::Session;

mod imp {
    use super::*;
    use glib::WeakRef;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-photo.ui")]
    pub(crate) struct MessagePhoto {
        pub(super) binding: RefCell<Option<gtk::ExpressionWatch>>,
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) old_message: WeakRef<glib::Object>,
        #[template_child]
        pub(super) media: TemplateChild<Media>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessagePhoto {
        const NAME: &'static str = "ContentMessagePhoto";
        type Type = super::MessagePhoto;
        type ParentType = MessageRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessagePhoto {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.connect_message_notify(|obj, _| obj.update_widget());
        }
    }

    impl WidgetImpl for MessagePhoto {}
}

glib::wrapper! {
    pub(crate) struct MessagePhoto(ObjectSubclass<imp::MessagePhoto>)
        @extends gtk::Widget, MessageRow;
}

impl MessagePhoto {
    fn update_widget(&self) {
        let imp = self.imp();

        if let Some(binding) = imp.binding.take() {
            binding.unwatch();
        }

        if let Some(old_message) = imp.old_message.upgrade() {
            if let Some(id) = imp.handler_id.take() {
                old_message.disconnect(id);
            }
        }

        let message = self.message().downcast::<Message>().unwrap();

        // Setup caption expression
        let caption_binding = Message::this_expression("content")
            .chain_closure::<String>(closure!(|_: Message, content: BoxedMessageContent| {
                if let MessageContent::MessagePhoto(data) = content.0 {
                    parse_formatted_text(data.caption)
                } else {
                    unreachable!();
                }
            }))
            .bind(&*imp.media, "caption", Some(&message));
        imp.binding.replace(Some(caption_binding));

        // Load photo
        let handler_id =
            message.connect_content_notify(clone!(@weak self as obj => move |message, _| {
                obj.update_photo(message);
            }));
        imp.handler_id.replace(Some(handler_id));
        self.update_photo(&message);

        imp.old_message.set(Some(self.message().as_ref()));
    }

    fn update_photo(&self, message: &Message) {
        if let MessageContent::MessagePhoto(data) = message.content().0 {
            if let Some(photo_size) = data.photo.sizes.last() {
                let imp = self.imp();

                // Reset media widget
                imp.media.set_paintable(None);
                imp.media
                    .set_aspect_ratio(photo_size.width as f64 / photo_size.height as f64);

                if photo_size.photo.local.is_downloading_completed {
                    imp.media.set_download_progress(1.0);
                    self.load_photo_from_path(&photo_size.photo.local.path);
                } else {
                    imp.media.set_download_progress(0.0);
                    self.download_photo(photo_size.photo.id, &message.chat().session());
                }
            }
        }
    }

    fn download_photo(&self, file_id: i32, session: &Session) {
        let (sender, receiver) = glib::MainContext::sync_channel::<File>(Default::default(), 5);

        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return glib::Continue(false), move |file| {
                if file.local.is_downloading_completed {
                    obj.imp().media.set_download_progress(1.0);
                    obj.load_photo_from_path(&file.local.path);
                } else {
                    let progress = file.local.downloaded_size as f64 / file.expected_size as f64;
                    obj.imp().media.set_download_progress(progress);
                }

                glib::Continue(true)
            }),
        );

        session.download_file(file_id, sender);
    }

    fn load_photo_from_path(&self, path: &str) {
        // TODO: Consider changing this to use an async api when
        // https://github.com/gtk-rs/gtk4-rs/pull/777 is merged
        let texture = gdk::Texture::from_filename(path).unwrap();
        self.imp().media.set_paintable(Some(texture.upcast()));
    }
}
