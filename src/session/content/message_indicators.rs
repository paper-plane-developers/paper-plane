use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::session::chat::Message;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-indicators.ui")]
    pub struct MessageIndicators {
        #[template_child]
        pub timestamp: TemplateChild<gtk::Label>,
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
    }
}
