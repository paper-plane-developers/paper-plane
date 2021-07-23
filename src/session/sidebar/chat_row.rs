use askama_escape::escape;
use gettextrs::gettext;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use tdgrand::enums::MessageContent;
use tdgrand::types::File;

use crate::session::chat::{BoxedMessageContent, Message};
use crate::session::Chat;

fn stringify_message_content(content: MessageContent) -> String {
    match content {
        MessageContent::MessageText(content) => {
            escape(&content.text.text, askama_escape::Html).to_string()
        }
        _ => format!("<i>{}</i>", gettext("This message is unsupported")),
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-chat-row.ui")]
    pub struct ChatRow {
        pub chat: RefCell<Option<Chat>>,
        pub photo_updated_handler: RefCell<Option<glib::SignalHandlerId>>,
        #[template_child]
        pub photo_avatar: TemplateChild<adw::Avatar>,
        #[template_child]
        pub last_message_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatRow {
        const NAME: &'static str = "SidebarChatRow";
        type Type = super::ChatRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "chat",
                    "Chat",
                    "The chat represented by this row",
                    Chat::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "chat" => {
                    let chat = value.get().unwrap();
                    obj.set_chat(chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ChatRow {}
    impl BinImpl for ChatRow {}
}

glib::wrapper! {
    pub struct ChatRow(ObjectSubclass<imp::ChatRow>)
        @extends gtk::Widget, adw::Bin;
}

impl ChatRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChatRow")
    }

    fn update_photo_avatar(&self, chat: &Chat) {
        let self_ = imp::ChatRow::from_instance(self);

        if let Some(photo) = chat.photo() {
            if photo.small.local.is_downloading_completed {
                let file = gio::File::for_path(photo.small.local.path);
                let photo = gdk::Texture::from_file(&file).unwrap();
                self_.photo_avatar.set_custom_image(Some(&photo));
                return;
            } else if photo.small.local.can_be_downloaded {
                let (sender, receiver) =
                    glib::MainContext::sync_channel::<File>(Default::default(), 5);
                receiver.attach(
                    None,
                    clone!(@weak chat => @default-return glib::Continue(false), move |file| {
                        let mut photo = chat.photo().unwrap();
                        photo.small = file;
                        chat.set_photo(Some(photo));

                        glib::Continue(true)
                    }),
                );

                chat.session().download_file(photo.small.id, sender);
            }
        }

        self_.photo_avatar.set_custom_image(None::<&gdk::Paintable>);
    }

    pub fn chat(&self) -> Option<Chat> {
        let self_ = imp::ChatRow::from_instance(self);
        self_.chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let self_ = imp::ChatRow::from_instance(self);

        // Remove signals from the previous chat
        if let Some(chat) = self_.chat.take() {
            if let Some(photo_updated_handler) = self_.photo_updated_handler.take() {
                chat.disconnect(photo_updated_handler);
            }
        }

        if let Some(ref chat) = chat {
            let chat_expression = gtk::ConstantExpression::new(&chat);
            let last_message_expression = gtk::PropertyExpression::new(
                Chat::static_type(),
                Some(&chat_expression),
                "last-message",
            );
            let content_expression = gtk::PropertyExpression::new(
                Message::static_type(),
                Some(&last_message_expression),
                "content",
            );
            let stringified_content_expression = gtk::ClosureExpression::new(
                move |expressions| -> String {
                    let content = expressions[1].get::<BoxedMessageContent>().unwrap();
                    stringify_message_content(content.0)
                },
                &[content_expression.upcast()],
            );

            let last_message_label = self_.last_message_label.get();
            stringified_content_expression.bind(
                &last_message_label,
                "label",
                Some(&last_message_label),
            );

            self.update_photo_avatar(chat);
            self_
                .photo_updated_handler
                .replace(Some(chat.connect_photo_updated(
                    clone!(@weak self as obj => move |chat| {
                        obj.update_photo_avatar(chat);
                    }),
                )));
        }

        self_.chat.replace(chat);
        self.notify("chat");
    }
}
