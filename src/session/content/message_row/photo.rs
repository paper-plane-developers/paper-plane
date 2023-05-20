use std::cell::RefCell;

use glib::clone;
use glib::closure;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use tdlib::enums::MessageContent;

use super::base::MessageBaseExt;
use crate::session::content::message_row::MediaPicture;
use crate::session::content::message_row::MessageBase;
use crate::session::content::message_row::MessageBaseImpl;
use crate::session::content::message_row::MessageBubble;
use crate::tdlib::BoxedMessageContent;
use crate::tdlib::Message;
use crate::utils::decode_image_from_path;
use crate::utils::parse_formatted_text;
use crate::utils::spawn;
use crate::Session;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/content-message-photo.ui")]
    pub(crate) struct MessagePhoto {
        pub(super) binding: RefCell<Option<gtk::ExpressionWatch>>,
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) message: RefCell<Option<Message>>,
        #[template_child]
        pub(super) message_bubble: TemplateChild<MessageBubble>,
        #[template_child]
        pub(super) picture: TemplateChild<MediaPicture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessagePhoto {
        const NAME: &'static str = "MessagePhoto";
        type Type = super::MessagePhoto;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessagePhoto {
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

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_scale_factor_notify(|obj| {
                obj.update_photo(obj.imp().message.borrow().as_ref().unwrap());
            });
        }
    }

    impl WidgetImpl for MessagePhoto {}
    impl MessageBaseImpl for MessagePhoto {}
}

glib::wrapper! {
    pub(crate) struct MessagePhoto(ObjectSubclass<imp::MessagePhoto>)
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessagePhoto {
    type Message = Message;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        if let Some(binding) = imp.binding.take() {
            binding.unwatch();
        }

        if let Some(old_message) = imp.message.take() {
            let handler_id = imp.handler_id.take().unwrap();
            old_message.disconnect(handler_id);
        }

        imp.message.replace(Some(message));

        let message_ref = imp.message.borrow();
        let message = message_ref.as_ref().unwrap();

        imp.message_bubble.update_from_message(message, true);

        // Setup caption expression
        let caption_binding = Message::this_expression("content")
            .chain_closure::<String>(closure!(|_: Message, content: BoxedMessageContent| {
                if let MessageContent::MessagePhoto(data) = content.0 {
                    parse_formatted_text(data.caption)
                } else {
                    unreachable!();
                }
            }))
            .bind(&*imp.message_bubble, "label", Some(message));
        imp.binding.replace(Some(caption_binding));

        // Load photo
        let handler_id =
            message.connect_content_notify(clone!(@weak self as obj => move |message, _| {
                obj.update_photo(message);
            }));
        imp.handler_id.replace(Some(handler_id));
        self.update_photo(message);

        self.notify("message");
    }
}

impl MessagePhoto {
    fn update_photo(&self, message: &Message) {
        if let MessageContent::MessagePhoto(mut data) = message.content().0 {
            let imp = self.imp();
            // Choose the right photo size based on the screen scale factor.
            // See https://core.telegram.org/api/files#image-thumbnail-types for more
            // information about photo sizes.
            let photo_size = if self.scale_factor() > 2 {
                data.photo.sizes.pop().unwrap()
            } else {
                let type_ = if self.scale_factor() > 1 { "y" } else { "x" };

                match data.photo.sizes.iter().position(|s| s.r#type == type_) {
                    Some(pos) => data.photo.sizes.swap_remove(pos),
                    None => data.photo.sizes.pop().unwrap(),
                }
            };

            imp.picture
                .set_aspect_ratio(photo_size.width as f64 / photo_size.height as f64);

            if photo_size.photo.local.is_downloading_completed {
                self.load_photo(photo_size.photo.local.path);
            } else {
                imp.picture.set_paintable(
                    data.photo
                        .minithumbnail
                        .and_then(|m| {
                            gdk::Texture::from_bytes(&glib::Bytes::from_owned(glib::base64_decode(
                                &m.data,
                            )))
                            .ok()
                        })
                        .as_ref(),
                );

                let file_id = photo_size.photo.id;
                let session = message.chat().session();
                spawn(clone!(@weak self as obj, @weak session => async move {
                    obj.download_photo(file_id, &session).await;
                }));
            }
        }
    }

    async fn download_photo(&self, file_id: i32, session: &Session) {
        match session.download_file(file_id).await {
            Ok(file) => {
                self.load_photo(file.local.path);
            }
            Err(e) => {
                log::warn!("Failed to download a photo: {e:?}");
            }
        }
    }

    fn load_photo(&self, path: String) {
        let message_id = self.message().id();

        spawn(clone!(@weak self as obj => async move {
            let result = gio::spawn_blocking(move || decode_image_from_path(&path))
                .await
                .unwrap();

            // Check if the current message id is the same as the one at
            // the time of the request. It may be changed because of the
            // ListView recycling while decoding the image.
            if obj.message().id() != message_id {
                return;
            }

            match result {
                Ok(texture) => {
                    obj.imp().picture.set_paintable(Some(&texture));
                }
                Err(e) => {
                    log::warn!("Error decoding a photo: {e:?}");
                }
            }
        }));
    }
}
