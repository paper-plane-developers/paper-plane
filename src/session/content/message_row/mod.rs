mod base;
mod bubble;
mod document;
mod indicators;
mod label;
mod media_picture;
mod photo;
mod sticker;
mod sticker_picture;
mod text;
mod video;

use self::base::{MessageBase, MessageBaseExt, MessageBaseImpl};
use self::bubble::MessageBubble;
use self::document::MessageDocument;
use self::indicators::MessageIndicators;
use self::label::MessageLabel;
use self::media_picture::MediaPicture;
use self::photo::MessagePhoto;
use self::sticker::MessageSticker;
use self::sticker_picture::StickerPicture;
use self::text::MessageText;
use self::video::MessageVideo;

use adw::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use tdlib::enums::{MessageContent, StickerFormat};

use crate::components::Avatar;
use crate::tdlib::{Chat, ChatType, Message, MessageForwardOrigin, MessageSender};
use crate::utils::spawn;

const AVATAR_SIZE: i32 = 32;
const SPACING: i32 = 6;
const VISIBLE_MESSAGE_DELAY_MILLIS: u64 = 100;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;
    use std::time::Duration;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    template MessageRow {
        GestureClick {
            button: 1;
            released => on_released() swapped;
        }
    }
    "#)]
    pub(crate) struct MessageRow {
        /// A `Message` or `SponsoredMessage`
        pub(super) message: RefCell<Option<glib::Object>>,
        pub(super) content: RefCell<Option<gtk::Widget>>,
        pub(super) avatar: RefCell<Option<Avatar>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "MessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.install_action("message-row.reply", None, move |widget, _, _| {
                widget.reply()
            });
            klass.install_action("message-row.edit", None, move |widget, _, _| widget.edit());
            klass.install_action("message-row.revoke-delete", None, move |widget, _, _| {
                widget.show_delete_dialog(true)
            });
            klass.install_action("message-row.delete", None, move |widget, _, _| {
                widget.show_delete_dialog(false)
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<glib::Object>("message")
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
            let obj = self.obj();

            match pspec.name() {
                "message" => obj.message().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            if let Some(avatar) = self.avatar.borrow().as_ref() {
                avatar.unparent();
            }

            if let Some(content) = self.content.borrow().as_ref() {
                content.unparent();
            }
        }
    }

    impl WidgetImpl for MessageRow {
        fn map(&self) {
            self.parent_map();

            let obj = self.obj();
            glib::timeout_add_local_once(
                Duration::from_millis(VISIBLE_MESSAGE_DELAY_MILLIS),
                clone!(@weak obj => move || if obj.is_mapped() {
                    if let Ok(message) = obj.message().downcast::<Message>() {
                        obj.activate_action(
                            "chat-history.add-visible-message",
                            Some(&message.id().to_variant()),
                        )
                        .unwrap();
                    }
                }),
            );
        }

        fn unmap(&self) {
            self.parent_unmap();

            let obj = self.obj();
            glib::timeout_add_local_once(
                Duration::from_millis(VISIBLE_MESSAGE_DELAY_MILLIS),
                clone!(@weak obj => move || if !obj.is_mapped() {
                    if let Ok(message) = obj.message().downcast::<Message>() {
                        obj.activate_action(
                            "chat-history.remove-visible-message",
                            Some(&message.id().to_variant()),
                        )
                        .unwrap();
                    }
                }),
            );
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk::Widget;
}

#[gtk::template_callbacks]
impl MessageRow {
    pub(crate) fn new(message: &glib::Object) -> Self {
        let layout_manager = gtk::BoxLayout::builder().spacing(SPACING).build();
        glib::Object::builder()
            .property("layout-manager", layout_manager)
            .property("message", message)
            .build()
    }

    #[template_callback]
    fn on_released(&self, n_press: i32, _x: f64, _y: f64) {
        if n_press == 2 && self.can_reply_to_message() {
            self.reply();
        }
    }

    fn reply(&self) {
        if let Ok(message) = self.message().downcast::<Message>() {
            self.activate_action("chat-history.reply", Some(&message.id().to_variant()))
                .unwrap();
        }
    }

    fn edit(&self) {
        if let Ok(message) = self.message().downcast::<Message>() {
            self.activate_action("chat-history.edit", Some(&message.id().to_variant()))
                .unwrap();
        }
    }

    fn show_delete_dialog(&self, revoke: bool) {
        let window: gtk::Window = self.root().and_then(|root| root.downcast().ok()).unwrap();

        let message = if revoke {
            gettext("Do you want to delete this message for <b>everyone</b>?")
        } else {
            gettext("Do you want to delete this message?")
        };

        let dialog = adw::MessageDialog::builder()
            .heading(gettext("Confirm Message Deletion"))
            .body_use_markup(true)
            .body(message)
            .transient_for(&window)
            .build();

        dialog.add_responses(&[("no", &gettext("_No")), ("yes", &gettext("_Yes"))]);
        dialog.set_default_response(Some("no"));
        dialog.set_response_appearance("yes", adw::ResponseAppearance::Destructive);

        dialog.choose(
            gio::Cancellable::NONE,
            clone!(@weak self as obj => move |response| {
                if response == "yes" {
                    if let Ok(message) = obj.message().downcast::<Message>() {
                        spawn(async move {
                            if let Err(e) = message.delete(revoke).await {
                                log::warn!("Error deleting a message (revoke = {}): {:?}", revoke, e);
                            }
                        });
                    }
                }
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

        self.update_content(message.clone());

        imp.message.replace(Some(message));

        // TODO: Update actions when needed (e.g. chat permissions change)
        self.update_actions();

        self.notify("message");
    }

    fn can_reply_to_message(&self) -> bool {
        if let Some(message) = self.message().downcast_ref::<Message>() {
            can_send_messages_in_chat(&message.chat())
        } else {
            false
        }
    }

    fn can_edit_message(&self) -> bool {
        if let Some(message) = self.message().downcast_ref::<Message>() {
            let is_text_message = matches!(message.content().0, MessageContent::MessageText(_));

            // TODO: Support more message types in the future
            is_text_message && message.can_be_edited() && can_send_messages_in_chat(&message.chat())
        } else {
            false
        }
    }

    fn update_actions(&self) {
        self.action_set_enabled("message-row.reply", self.can_reply_to_message());
        self.action_set_enabled("message-row.edit", self.can_edit_message());

        if let Some(message) = self.message().downcast_ref::<Message>() {
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
            // Do not mark channel messages as outgoing
            let is_outgoing = match message_.chat().type_() {
                ChatType::Supergroup(data) if data.is_channel() => false,
                _ => message_.is_outgoing(),
            };

            match message_.content().0 {
                // FIXME: Re-enable MessageVideo when
                // https://github.com/melix99/telegrand/issues/410 is fixed
                MessageContent::MessageAnimation(_) /*| MessageContent::MessageVideo(_)*/ => {
                    self.update_specific_content::<_, MessageVideo>(message_.clone());
                }
                MessageContent::MessagePhoto(_) => {
                    self.update_specific_content::<_, MessagePhoto>(message_.clone());
                }
                MessageContent::MessageSticker(data)
                    if data.sticker.format == StickerFormat::Webp =>
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
        } else {
            content.set_halign(gtk::Align::Start);
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

fn can_send_messages_in_chat(chat: &Chat) -> bool {
    use tdlib::enums::ChatMemberStatus::*;
    let member_status = match chat.type_() {
        ChatType::Supergroup(supergroup) => Some(supergroup.status()),
        ChatType::BasicGroup(supergroup) => Some(supergroup.status()),
        _ => None,
    };
    member_status
        .map(|s| match s.0 {
            Creator(_) => true,
            Administrator(_) => true,
            Member => chat.permissions().0.can_send_messages,
            Restricted(data) => {
                chat.permissions().0.can_send_messages && data.permissions.can_send_messages
            }
            Left => false,
            Banned(_) => false,
        })
        .unwrap_or(true)
}
