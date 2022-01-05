use glib::clone;
use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums::MessageContent, types::File};

use crate::session::chat::{BoxedMessageContent, Message};
use crate::session::content::{message_row::Media, MessageRow, MessageRowExt};
use crate::utils::parse_formatted_text;
use crate::Session;

mod imp {
    use super::*;
    use glib::WeakRef;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-photo.ui")]
    pub struct MessagePhoto {
        pub binding: RefCell<Option<gtk::ExpressionWatch>>,
        pub handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub old_message: WeakRef<glib::Object>,
        #[template_child]
        pub media: TemplateChild<Media>,
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
    pub struct MessagePhoto(ObjectSubclass<imp::MessagePhoto>)
        @extends gtk::Widget, MessageRow;
}

impl MessagePhoto {
    fn update_widget(&self) {
        let self_ = imp::MessagePhoto::from_instance(self);

        if let Some(binding) = self_.binding.take() {
            binding.unwatch();
        }

        if let Some(old_message) = self_.old_message.upgrade() {
            if let Some(id) = self_.handler_id.take() {
                old_message.disconnect(id);
            }
        }

        if let Some(message) = self.message() {
            let message = message.downcast_ref::<Message>().unwrap();

            // Setup caption expression
            let content_expression = gtk::PropertyExpression::new(
                Message::static_type(),
                gtk::NONE_EXPRESSION,
                "content",
            );
            let text_expression = gtk::ClosureExpression::new(
                |args| {
                    let content = args[1].get::<BoxedMessageContent>().unwrap();
                    if let MessageContent::MessagePhoto(data) = content.0 {
                        parse_formatted_text(data.caption)
                    } else {
                        unreachable!();
                    }
                },
                &[content_expression.upcast()],
            );
            let text_binding = text_expression.bind(&*self_.media, "caption", Some(message));
            self_.binding.replace(Some(text_binding));

            // Load photo
            let handler_id =
                message.connect_content_notify(clone!(@weak self as obj => move |message, _| {
                    obj.update_photo(message);
                }));
            self_.handler_id.replace(Some(handler_id));
            self.update_photo(message);
        }

        self_.old_message.set(self.message().as_ref());
    }

    fn update_photo(&self, message: &Message) {
        if let MessageContent::MessagePhoto(data) = message.content().0 {
            if let Some(photo_size) = data.photo.sizes.last() {
                let self_ = imp::MessagePhoto::from_instance(self);

                // Reset media widget
                self_.media.set_paintable(None);
                self_
                    .media
                    .set_aspect_ratio(photo_size.width as f64 / photo_size.height as f64);

                if photo_size.photo.local.is_downloading_completed {
                    self_.media.set_download_progress(1.0);
                    self.load_photo_from_path(&photo_size.photo.local.path);
                } else {
                    self_.media.set_download_progress(0.0);
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
                let self_ = imp::MessagePhoto::from_instance(&obj);

                if file.local.is_downloading_completed {
                    self_.media.set_download_progress(1.0);
                    obj.load_photo_from_path(&file.local.path);
                } else {
                    let progress = file.local.downloaded_size as f64 / file.expected_size as f64;
                    self_.media.set_download_progress(progress);
                }

                glib::Continue(true)
            }),
        );

        session.download_file(file_id, sender);
    }

    fn load_photo_from_path(&self, path: &str) {
        let self_ = imp::MessagePhoto::from_instance(self);
        // TODO: Consider changing this to use an async api when
        // https://github.com/gtk-rs/gtk4-rs/pull/777 is merged
        let file = gio::File::for_path(path);
        self_
            .media
            .set_paintable(Some(gdk::Texture::from_file(&file).unwrap().upcast()));
    }
}
