use gettextrs::gettext;
use glib::closure;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::MessageContent;

use super::base::MessageBaseExt;
use super::{MessageBase, MessageBaseImpl, MessageBubble};
use crate::tdlib::{BoxedMessageContent, Message, SponsoredMessage};
use crate::utils::parse_formatted_text;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-text.ui")]
    pub(crate) struct MessageText {
        pub(super) bindings: RefCell<Vec<gtk::ExpressionWatch>>,
        pub(super) message: RefCell<Option<glib::Object>>,
        #[template_child]
        pub(super) message_bubble: TemplateChild<MessageBubble>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageText {
        const NAME: &'static str = "MessageText";
        type Type = super::MessageText;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageText {
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
            match pspec.name() {
                "message" => self.message.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageText {}
    impl MessageBaseImpl for MessageText {}
}

glib::wrapper! {
    pub(crate) struct MessageText(ObjectSubclass<imp::MessageText>)
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessageText {
    type Message = glib::Object;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        let mut bindings = imp.bindings.borrow_mut();

        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        if let Some(message) = message.downcast_ref::<Message>() {
            imp.message_bubble.update_from_message(message, false);

            // Set content label expression
            let text_binding = Message::this_expression("content")
                .chain_closure::<String>(closure!(|_: Message, content: BoxedMessageContent| {
                    format_message_content_text(content.0)
                }))
                .bind(&*imp.message_bubble, "label", Some(message));
            bindings.push(text_binding);
        } else if let Some(sponsored_message) = message.downcast_ref::<SponsoredMessage>() {
            imp.message_bubble
                .update_from_sponsored_message(sponsored_message);

            // Set content label expression
            let text_binding = SponsoredMessage::this_expression("content")
                .chain_closure::<String>(closure!(
                    |_: SponsoredMessage, content: BoxedMessageContent| {
                        format_message_content_text(content.0)
                    }
                ))
                .bind(&*imp.message_bubble, "label", Some(sponsored_message));
            bindings.push(text_binding);
        } else {
            unreachable!("Unexpected message type: {:?}", message);
        }

        imp.message.replace(Some(message));
        self.notify("message");
    }
}

fn format_message_content_text(content: MessageContent) -> String {
    match content {
        MessageContent::MessageText(content) => parse_formatted_text(content.text),
        _ => format!("<i>{}</i>", gettext("This message is unsupported")),
    }
}
