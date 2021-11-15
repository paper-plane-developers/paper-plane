use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::enums::MessageSendingState;

use crate::session::chat::{Chat, Message};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-indicators.ui")]
    pub struct MessageIndicators {
        #[template_child]
        pub timestamp: TemplateChild<gtk::Label>,
        #[template_child]
        pub status_stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageIndicators {
        const NAME: &'static str = "ContentMessageIndicators";
        type Type = super::MessageIndicators;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageIndicators {
        fn dispose(&self, _obj: &Self::Type) {
            self.timestamp.unparent();
            self.status_stack.unparent();
        }
    }

    impl WidgetImpl for MessageIndicators {}
}

glib::wrapper! {
    pub struct MessageIndicators(ObjectSubclass<imp::MessageIndicators>)
        @extends gtk::Widget;
}

impl Default for MessageIndicators {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageIndicators {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageIndicators")
    }

    pub fn set_message(&self, message: &Message) {
        let self_ = imp::MessageIndicators::from_instance(self);

        let message_expression = gtk::ConstantExpression::new(message);
        let date_expression =
            gtk::PropertyExpression::new(Message::static_type(), Some(&message_expression), "date");
        let timestamp_expression = gtk::ClosureExpression::new(
            move |args| -> String {
                let date = args[1].get::<i32>().unwrap();
                let datetime = glib::DateTime::from_unix_utc(date as i64)
                    .and_then(|t| t.to_local())
                    .unwrap();
                let mut time = datetime.format("%X").unwrap().to_string();

                // Remove seconds
                time.replace_range(5..8, "");
                time
            },
            &[date_expression.upcast()],
        );
        timestamp_expression.bind(&*self_.timestamp, "label", gtk::NONE_WIDGET);

        // Message Status
        let message_status_visibility_expression = gtk::PropertyExpression::new(
            Message::static_type(),
            Some(&message_expression),
            "is-outgoing",
        );
        message_status_visibility_expression.bind(
            &*self_.status_stack,
            "visible",
            gtk::NONE_WIDGET,
        );
        let last_read_expression = gtk::PropertyExpression::new(
            Chat::static_type(),
            Some(&gtk::ConstantExpression::new(message.chat())),
            "last-read-outbox-message-id",
        );
        let status_icon_expression = gtk::ClosureExpression::new(
            move |args| {
                let message = args[1].get::<Message>().unwrap();

                if message.is_outgoing() {
                    // TODO: Always set messages in "Saved Messages" chat as 'read'.
                    match message.sending_state() {
                        Some(state) => match state {
                            MessageSendingState::Failed(_) => "failed",
                            MessageSendingState::Pending => "pending",
                        },
                        None => {
                            if message.id() <= args[2].get::<i64>().unwrap() {
                                "read"
                            } else {
                                "unread"
                            }
                        }
                    }
                } else {
                    "empty"
                }
            },
            &[message_expression.upcast(), last_read_expression.upcast()],
        );
        status_icon_expression.bind(&*self_.status_stack, "visible-child-name", gtk::NONE_WIDGET);
    }
}
