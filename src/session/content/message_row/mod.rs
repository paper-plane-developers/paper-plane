mod media;
mod media_picture;
mod photo;
mod sticker;
mod sticker_picture;
mod text;

use self::media::Media;
use self::media_picture::MediaPicture;
pub(crate) use self::photo::MessagePhoto;
pub(crate) use self::sticker::MessageSticker;
use self::sticker_picture::StickerPicture;
pub(crate) use self::text::MessageText;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::session::chat::{Message, MessageForwardOrigin, MessageSender, SponsoredMessage};
use crate::session::components::Avatar;
use crate::session::ChatType;

const AVATAR_SIZE: i32 = 32;
const SPACING: i32 = 6;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct MessageRow {
        /// A `Message` or `SponsoredMessage`
        pub(super) message: RefCell<Option<glib::Object>>,
        pub(super) content: RefCell<Option<gtk::Widget>>,
        pub(super) avatar: RefCell<Option<Avatar>>,
        pub(super) is_outgoing: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "ContentMessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for MessageRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "message",
                        "Message",
                        "The message represented by this row",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "content",
                        "Content",
                        "The content widget",
                        gtk::Widget::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
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
                "message" => obj.set_message(value.get().unwrap()),
                "content" => obj.set_content(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => obj.message().to_value(),
                "content" => obj.content().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            if let Some(avatar) = self.avatar.borrow().as_ref() {
                avatar.unparent();
            }

            if let Some(content) = self.content.borrow().as_ref() {
                content.unparent();
            }
        }
    }

    impl WidgetImpl for MessageRow {}
}

glib::wrapper! {
    pub(crate) struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk::Widget;
}

pub(crate) trait MessageRowExt: IsA<MessageRow> {
    fn new(message: &glib::Object) -> Self;

    fn message(&self) -> glib::Object {
        self.upcast_ref().imp().message.borrow().clone().unwrap()
    }

    fn set_message(&self, message: glib::Object) {
        let imp = self.upcast_ref().imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        if let Some(message) = message.downcast_ref::<Message>() {
            imp.is_outgoing.set(message.is_outgoing());

            let show_avatar = if message.is_outgoing() {
                false
            } else if message.chat().is_own_chat() {
                message.forward_info().is_some()
            } else {
                match message.chat().type_() {
                    ChatType::BasicGroup(_) => true,
                    ChatType::Supergroup(supergroup) => !supergroup.is_channel(),
                    _ => false,
                }
            };

            if show_avatar {
                let avatar = {
                    let mut avatar_borrow = imp.avatar.borrow_mut();
                    if let Some(avatar) = avatar_borrow.clone() {
                        avatar
                    } else {
                        let avatar = Avatar::new();
                        avatar.set_size(AVATAR_SIZE);
                        avatar.set_valign(gtk::Align::End);

                        // Insert at the beginning
                        avatar.insert_after(self.upcast_ref(), gtk::Widget::NONE);

                        *avatar_borrow = Some(avatar.clone());
                        avatar
                    }
                };

                if message.chat().is_own_chat() {
                    match message.forward_info().unwrap().origin() {
                        MessageForwardOrigin::User(user) => {
                            avatar.set_custom_text(None);
                            avatar.set_item(Some(user.clone().upcast()));
                        }
                        MessageForwardOrigin::Chat { chat, .. }
                        | MessageForwardOrigin::Channel { chat, .. } => {
                            avatar.set_custom_text(None);
                            avatar.set_item(Some(chat.clone().upcast()));
                        }
                        MessageForwardOrigin::HiddenUser { sender_name }
                        | MessageForwardOrigin::MessageImport { sender_name } => {
                            avatar.set_item(None);
                            avatar.set_custom_text(Some(sender_name));
                        }
                    }
                } else {
                    let avatar_item = match message.sender() {
                        MessageSender::User(user) => user.clone().upcast(),
                        MessageSender::Chat(chat) => chat.clone().upcast(),
                    };
                    avatar.set_custom_text(None);
                    avatar.set_item(Some(avatar_item));
                }
            } else {
                if let Some(avatar) = imp.avatar.borrow().as_ref() {
                    avatar.unparent();
                }
                imp.avatar.replace(None);
            }
        } else if message.downcast_ref::<SponsoredMessage>().is_some() {
            imp.is_outgoing.set(false);
        } else {
            unreachable!("Unexpected message type: {:?}", message);
        }

        if let Some(content) = imp.content.borrow().as_ref() {
            if imp.is_outgoing.get() {
                content.set_halign(gtk::Align::End);
                content.set_margin_start(AVATAR_SIZE + SPACING);
                content.set_margin_end(0);
                content.add_css_class("outgoing");
            } else {
                content.set_halign(gtk::Align::Start);
                content.set_margin_start(0);
                content.set_margin_end(AVATAR_SIZE + SPACING);
                content.remove_css_class("outgoing");
            }
        }

        imp.message.replace(Some(message));
        self.notify("message");
    }

    fn connect_message_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("message"), f)
    }

    fn content(&self) -> Option<gtk::Widget> {
        self.upcast_ref().imp().content.borrow().to_owned()
    }

    fn set_content(&self, content: Option<gtk::Widget>) {
        if self.content() == content {
            return;
        }

        let imp = self.upcast_ref().imp();

        if let Some(content) = imp.content.borrow().as_ref() {
            content.unparent();
        }

        if let Some(ref content) = content {
            content.set_hexpand(true);
            content.set_valign(gtk::Align::Start);

            // Insert at the end
            content.insert_before(self.upcast_ref(), gtk::Widget::NONE);
        }

        imp.content.replace(content);
        self.notify("content");
    }
}

impl<T: glib::object::IsClass + IsA<glib::Object> + IsA<MessageRow>> MessageRowExt for T {
    fn new(message: &glib::Object) -> Self {
        let layout_manager = gtk::BoxLayout::builder().spacing(SPACING).build();
        glib::Object::new(&[("layout-manager", &layout_manager), ("message", message)])
            .expect("Failed to create MessageRow")
    }
}

unsafe impl<T: WidgetImpl + ObjectImpl + 'static> IsSubclassable<T> for MessageRow {
    fn class_init(class: &mut glib::Class<Self>) {
        <gtk::Widget as IsSubclassable<T>>::class_init(class.upcast_ref_mut());
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <gtk::Widget as IsSubclassable<T>>::instance_init(instance);
    }
}
