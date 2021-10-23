use gettextrs::gettext;
use gtk::{glib, pango, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::enums::MessageContent;

use crate::session::chat::{BoxedMessageContent, Message};
use crate::session::content::MessageIndicators;
use crate::utils::parse_formatted_text;

const INDICATORS_PLACEHOLDER: char = '\u{200C}';

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-label.ui")]
    pub struct MessageLabel {
        pub indicators_size: RefCell<Option<(i32, i32)>>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub indicators: TemplateChild<MessageIndicators>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageLabel {
        const NAME: &'static str = "ContentMessageLabel";
        type Type = super::MessageLabel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageLabel {
        fn dispose(&self, _obj: &Self::Type) {
            self.label.unparent();
            self.indicators.unparent();
        }
    }

    impl WidgetImpl for MessageLabel {
        fn measure(
            &self,
            widget: &Self::Type,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let (_, natural_size) = self.indicators.preferred_size();

            if let Some(indicators_size) = self.indicators_size.borrow().as_ref() {
                if natural_size.width != indicators_size.0
                    || natural_size.height != indicators_size.1
                {
                    widget.update_label_attributes(&natural_size);
                }
            } else {
                widget.update_label_attributes(&natural_size);
            }

            self.indicators_size
                .replace(Some((natural_size.width, natural_size.height)));

            self.label.measure(orientation, for_size)
        }

        fn size_allocate(&self, _widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.label.allocate(width, height, baseline, None);

            let (_, natural_size) = self.indicators.preferred_size();
            let allocation = gtk::Allocation {
                x: width - natural_size.width,
                y: height - natural_size.height,
                width: natural_size.width,
                height: natural_size.height,
            };

            self.indicators.size_allocate(&allocation, -1);
        }

        fn request_mode(&self, _widget: &Self::Type) -> gtk::SizeRequestMode {
            self.label.request_mode()
        }
    }
}

glib::wrapper! {
    pub struct MessageLabel(ObjectSubclass<imp::MessageLabel>)
        @extends gtk::Widget;
}

impl Default for MessageLabel {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageLabel {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageLabel")
    }

    fn update_label_attributes(&self, indicators_size: &gtk::Requisition) {
        let self_ = imp::MessageLabel::from_instance(self);
        let attrs = pango::AttrList::new();
        let width = indicators_size.width;
        let height = indicators_size.height;
        let logical_rect = pango::Rectangle::new(
            0,
            -(height - (height / 4)) * pango::SCALE,
            width * pango::SCALE,
            height * pango::SCALE,
        );

        let mut shape = pango::Attribute::new_shape(&logical_rect, &logical_rect);
        let direction = pango::find_base_dir(&self_.label.text());

        if let pango::Direction::Rtl = direction {
            shape.set_start_index(0);
            shape.set_end_index(1);
        } else {
            shape.set_start_index((self_.label.text().len() - 1) as u32);
            shape.set_end_index(self_.label.text().len() as u32);
        }

        attrs.insert(shape);
        self_.label.set_attributes(Some(&attrs));
    }

    pub fn set_message(&self, message: &Message) {
        let self_ = imp::MessageLabel::from_instance(self);
        self_.indicators.set_message(message);

        let message_expression = gtk::ConstantExpression::new(message);
        let content_expression = gtk::PropertyExpression::new(
            Message::static_type(),
            Some(&message_expression),
            "content",
        );
        let text_expression = gtk::ClosureExpression::new(
            move |args| -> String {
                let content = args[1].get::<BoxedMessageContent>().unwrap();
                let text = format_message_content_text(content.0);
                format!("{}{}", text, INDICATORS_PLACEHOLDER)
            },
            &[content_expression.upcast()],
        );
        text_expression.bind(&*self_.label, "label", gtk::NONE_WIDGET);
    }
}

fn format_message_content_text(content: MessageContent) -> String {
    match content {
        MessageContent::MessageText(content) => parse_formatted_text(content.text),
        _ => format!("<i>{}</i>", gettext("This message is unsupported")),
    }
}
