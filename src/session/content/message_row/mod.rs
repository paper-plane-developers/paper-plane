mod base;
mod document;
mod indicators;
mod indicators_model;
mod label;
mod media;
mod media_picture;
mod photo;
mod sticker;
mod sticker_picture;
mod text;

use self::base::{MessageBase, MessageBaseExt, MessageBaseImpl};
use self::document::MessageDocument;
use self::indicators::MessageIndicators;
use self::label::MessageLabel;
use self::media::Media;
use self::media_picture::MediaPicture;
use self::photo::MessagePhoto;
use self::sticker::MessageSticker;
use self::sticker_picture::StickerPicture;
use self::text::MessageText;

use gettextrs::gettext;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::{MessageContent, StickerType};

use crate::session::components::Avatar;
use crate::tdlib::{ChatType, Message, MessageForwardOrigin, MessageSender};
use crate::utils::spawn;

const AVATAR_SIZE: i32 = 32;
const SPACING: i32 = 6;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub(crate) struct MessageRow {
        /// A `Message` or `SponsoredMessage`
        pub(super) message: RefCell<Option<glib::Object>>,
        pub(super) content: RefCell<Option<gtk::Widget>>,
        pub(super) avatar: RefCell<Option<Avatar>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "ContentMessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("message-row.revoke-delete", None, move |widget, _, _| {
                widget.show_delete_dialog(true)
            });
            klass.install_action("message-row.delete", None, move |widget, _, _| {
                widget.show_delete_dialog(false)
            });
        }
    }

    impl ObjectImpl for MessageRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "message",
                    "Message",
                    "The message represented by this row",
                    glib::Object::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "message" => obj.set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => obj.message().to_value(),
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

impl MessageRow {
    pub(crate) fn new(message: &glib::Object) -> Self {
        let layout_manager = gtk::BoxLayout::builder().spacing(SPACING).build();
        glib::Object::new(&[("layout-manager", &layout_manager), ("message", message)])
            .expect("Failed to create MessageRow")
    }

    fn show_delete_dialog(&self, revoke: bool) {
        let window: Option<gtk::Window> = self.root().and_then(|root| root.downcast().ok());
        let message = if revoke {
            gettext("Do you want to delete this message for everyone?")
        } else {
            gettext("Do you want to delete this message?")
        };
        let dialog = gtk::MessageDialog::new(
            window.as_ref(),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Warning,
            gtk::ButtonsType::YesNo,
            &message,
        );

        dialog.run_async(clone!(@weak self as obj => move |dialog, response| {
            if matches!(response, gtk::ResponseType::Yes) {
                if let Ok(message) = obj.message().downcast::<Message>() {
                    spawn(async move {
                        if let Err(e) = message.delete(revoke).await {
                            log::warn!("Error deleting a message (revoke = {}): {:?}", revoke, e);
                        }
                    });
                }
            }
            dialog.close();
        }));
    }

    pub(crate) fn message(&self) -> glib::Object {
        self.imp().message.borrow().clone().unwrap()
    }

    pub(crate) fn set_message(&self, message: glib::Object) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        if let Some(message) = message.downcast_ref::<Message>() {
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
                        avatar.insert_after(self, gtk::Widget::NONE);

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
        }

        self.update_actions(&message);
        self.update_content(message.clone());

        imp.message.replace(Some(message));
        self.notify("message");
    }

    fn update_actions(&self, message: &glib::Object) {
        if let Some(message) = message.downcast_ref::<Message>() {
            self.action_set_enabled("message-row.delete", message.can_be_deleted_only_for_self());
            self.action_set_enabled(
                "message-row.revoke-delete",
                message.can_be_deleted_for_all_users(),
            );
        } else {
            self.action_set_enabled("message-row.delete", false);
            self.action_set_enabled("message-row.revoke-delete", false);
        }
    }

    fn update_content(&self, message: glib::Object) {
        let is_outgoing = if let Some(message_) = message.downcast_ref::<Message>() {
            let is_outgoing = message_.is_outgoing();

            match message_.content().0 {
                MessageContent::MessagePhoto(_) => {
                    self.update_specific_content::<_, MessagePhoto>(message_.clone());
                }
                MessageContent::MessageSticker(data)
                    if matches!(
                        data.sticker.r#type,
                        StickerType::Static | StickerType::Mask(_)
                    ) =>
                {
                    self.update_specific_content::<_, MessageSticker>(message_.clone());
                }
                MessageContent::MessageDocument(_) => {
                    self.update_specific_content::<_, MessageDocument>(message_.clone());
                }
                _ => {
                    self.update_specific_content::<_, MessageText>(message);
                }
            }

            is_outgoing
        } else {
            self.update_specific_content::<_, MessageText>(message);
            false
        };

        let content_ref = self.imp().content.borrow();
        let content = content_ref.as_ref().unwrap();

        if is_outgoing {
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

    fn update_specific_content<M, B>(&self, message: M)
    where
        B: MessageBaseExt<Message = M>,
    {
        let mut content_ref = self.imp().content.borrow_mut();
        match content_ref.as_ref().and_then(|c| c.downcast_ref::<B>()) {
            Some(content) => {
                content.set_message(message);
            }
            None => {
                if let Some(old_content) = &*content_ref {
                    old_content.unparent();
                }

                let content = B::new(&message);
                content.set_hexpand(true);
                content.set_valign(gtk::Align::Start);

                // Insert at the end
                content.insert_before(self, gtk::Widget::NONE);

                *content_ref = Some(content.upcast());
            }
        }
    }
}
