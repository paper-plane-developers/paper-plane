use std::cell::RefCell;
use std::sync::OnceLock;

use gettextrs::gettext;
use glib::closure;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;
use crate::ui::MessageBaseExt;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/message_row/text.ui")]
    pub(crate) struct MessageText {
        pub(super) bindings: RefCell<Vec<gtk::ExpressionWatch>>,
        pub(super) message: glib::WeakRef<glib::Object>,
        #[template_child]
        pub(super) message_bubble: TemplateChild<ui::MessageBubble>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageText {
        const NAME: &'static str = "PaplMessageText";
        type Type = super::MessageText;
        type ParentType = ui::MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageText {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecObject::builder::<glib::Object>("message")
                    .explicit_notify()
                    .build()]
            })
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
                "message" => self.message.upgrade().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageText {}
    impl ui::MessageBaseImpl for MessageText {}
}

glib::wrapper! {
    pub(crate) struct MessageText(ObjectSubclass<imp::MessageText>)
        @extends gtk::Widget, ui::MessageBase;
}

impl ui::MessageBaseExt for MessageText {
    type Message = glib::Object;

    fn set_message(&self, message: &Self::Message) {
        let imp = self.imp();

        if imp.message.upgrade().as_ref() == Some(message) {
            return;
        }

        let mut bindings = imp.bindings.borrow_mut();

        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        if let Some(message) = message.downcast_ref::<model::Message>() {
            imp.message_bubble.update_from_message(message, false);

            // Set content label expression
            let text_binding = model::Message::this_expression("content")
                .chain_closure::<String>(closure!(
                    |_: model::Message, content: model::BoxedMessageContent| {
                        format_message_content_text(content.0)
                    }
                ))
                .bind(&*imp.message_bubble, "label", Some(message));
            bindings.push(text_binding);
        } else if let Some(sponsored_message) = message.downcast_ref::<model::SponsoredMessage>() {
            imp.message_bubble
                .update_from_sponsored_message(sponsored_message);

            // Set content label expression
            let text_binding = model::SponsoredMessage::this_expression("content")
                .chain_closure::<String>(closure!(
                    |_: model::SponsoredMessage, content: model::BoxedMessageContent| {
                        format_message_content_text(content.0)
                    }
                ))
                .bind(&*imp.message_bubble, "label", Some(sponsored_message));
            bindings.push(text_binding);
        } else {
            unreachable!("Unexpected message type: {:?}", message);
        }

        imp.message.set(Some(message));
        self.notify("message");
    }
}

fn format_message_content_text(content: tdlib::enums::MessageContent) -> String {
    match content {
        tdlib::enums::MessageContent::MessageText(content) => {
            utils::parse_formatted_text(content.text)
        }
        _ => format!("<i>{}</i>", gettext("This message is unsupported")),
    }
}
