use adw::{prelude::BinExt, subclass::prelude::BinImpl};
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::enums::ChatType;

use crate::session::chat::{Message, MessageSender};
use crate::session::components::Avatar;
use crate::session::content::MessageBubble;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-row.ui")]
    pub struct MessageRow {
        #[template_child]
        pub avatar_bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub content_bin: TemplateChild<adw::Bin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "ContentMessageRow";
        type Type = super::MessageRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageRow {}
    impl WidgetImpl for MessageRow {}
    impl BinImpl for MessageRow {}
}

glib::wrapper! {
    pub struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for MessageRow {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageRow")
    }

    pub fn set_message(&self, message: &Message) {
        let self_ = imp::MessageRow::from_instance(self);

        // Align message based on whether the message is outgoing or not
        if message.is_outgoing() {
            self_.avatar_bin.set_hexpand(true);
        } else {
            self_.avatar_bin.set_hexpand(false);
        }

        // Show avatar, if needed
        let show_avatar = {
            if !message.is_outgoing() {
                match message.chat().type_() {
                    ChatType::BasicGroup(_) => true,
                    ChatType::Supergroup(data) => !data.is_channel,
                    _ => false,
                }
            } else {
                false
            }
        };
        if show_avatar {
            let avatar = if let Some(Ok(avatar)) =
                self_.avatar_bin.child().map(|w| w.downcast::<Avatar>())
            {
                avatar
            } else {
                let avatar = Avatar::new();
                avatar.set_size(32);
                avatar.set_valign(gtk::Align::End);
                self_.avatar_bin.set_child(Some(&avatar));
                avatar
            };
            match message.sender() {
                MessageSender::User(user) => avatar.set_item(Some(user.avatar().clone())),
                MessageSender::Chat(chat) => avatar.set_item(Some(chat.avatar().clone())),
            }
        } else {
            self_.avatar_bin.set_child(None::<&gtk::Widget>);
        }

        // Show content widget
        let content = if let Some(Ok(content)) = self_
            .content_bin
            .child()
            .map(|w| w.downcast::<MessageBubble>())
        {
            content
        } else {
            let content = MessageBubble::new();
            self_.content_bin.set_child(Some(&content));
            content
        };
        content.set_message(message);
    }
}
