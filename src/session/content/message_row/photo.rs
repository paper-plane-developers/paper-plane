use glib::{clone, closure};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};
use tdlib::enums::MessageContent;
use tdlib::types::File;

use crate::session::content::message_row::{
    MediaPicture, MessageBase, MessageBaseImpl, MessageBubble,
};
use crate::session::content::ChatHistory;
use crate::tdlib::{BoxedMessageContent, Message};
use crate::utils::parse_formatted_text;
use crate::Session;

use super::base::MessageBaseExt;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-photo.ui")]
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
        const NAME: &'static str = "ContentMessagePhoto";
        type Type = super::MessagePhoto;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
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

        imp.message_bubble.update_from_message(&message, true);

        // Setup caption expression
        let caption_binding = Message::this_expression("content")
            .chain_closure::<String>(closure!(|_: Message, content: BoxedMessageContent| {
                if let MessageContent::MessagePhoto(data) = content.0 {
                    parse_formatted_text(data.caption)
                } else {
                    unreachable!();
                }
            }))
            .bind(&*imp.message_bubble, "label", Some(&message));
        imp.binding.replace(Some(caption_binding));

        // Load photo
        let handler_id =
            message.connect_content_notify(clone!(@weak self as obj => move |message, _| {
                obj.update_photo(message);
            }));
        imp.handler_id.replace(Some(handler_id));
        self.update_photo(&message);

        imp.message.replace(Some(message));
        self.notify("message");
    }
}

#[gtk::template_callbacks]
impl MessagePhoto {
    #[template_callback]
    fn on_released(&self) {
        let chat_history = self.ancestor(ChatHistory::static_type()).unwrap();
        let chat = chat_history
            .downcast_ref::<ChatHistory>()
            .unwrap()
            .chat()
            .unwrap();
        chat.session()
            .open_media(self.message(), &*self.imp().picture);
    }

    fn update_photo(&self, message: &Message) {
        if let MessageContent::MessagePhoto(data) = message.content().0 {
            let imp = self.imp();
            // Choose the right photo size based on the screen scale factor.
            // See https://core.telegram.org/api/files#image-thumbnail-types for more
            // information about photo sizes.
            let photo_size = if self.scale_factor() > 2 {
                data.photo.sizes.last().unwrap()
            } else {
                let type_ = if self.scale_factor() > 1 { "y" } else { "x" };
                data.photo
                    .sizes
                    .iter()
                    .find(|s| s.r#type == type_)
                    .unwrap_or_else(|| data.photo.sizes.last().unwrap())
            };

            imp.picture
                .set_aspect_ratio(photo_size.width as f64 / photo_size.height as f64);

            if photo_size.photo.local.is_downloading_completed {
                self.load_photo_from_path(&photo_size.photo.local.path);
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

                self.download_photo(photo_size.photo.id, &message.chat().session());
            }
        }
    }

    fn download_photo(&self, file_id: i32, session: &Session) {
        let (sender, receiver) = glib::MainContext::sync_channel::<File>(Default::default(), 5);

        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return glib::Continue(false), move |file| {
                if file.local.is_downloading_completed {
                    obj.load_photo_from_path(&file.local.path);
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
        self.imp().picture.set_paintable(Some(&texture));
    }
}
