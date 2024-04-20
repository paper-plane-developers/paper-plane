use std::cell::RefCell;
use std::sync::OnceLock;

use gtk::glib;
use gtk::pango;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::ui;

const OBJECT_REPLACEMENT_CHARACTER: char = '\u{FFFC}';
const INDICATORS_SPACING: i32 = 6;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/message_row/label.ui")]
    pub(crate) struct MessageLabel {
        pub(super) text: RefCell<String>,
        pub(super) indicators: RefCell<Option<ui::MessageIndicators>>,
        pub(super) indicators_size: RefCell<Option<(i32, i32)>>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageLabel {
        const NAME: &'static str = "PaplMessageLabel";
        type Type = super::MessageLabel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("messagelabel");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageLabel {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("label")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<ui::MessageIndicators>("indicators")
                        .construct_only()
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "label" => obj.set_label(value.get().unwrap()),
                "indicators" => obj.set_indicators(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "label" => obj.label().to_value(),
                "indicators" => obj.indicators().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            self.label.unparent();
            if let Some(indicators) = self.indicators.take() {
                indicators.unparent();
            }
        }
    }

    impl WidgetImpl for MessageLabel {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let obj = self.obj();

            if let Some(indicators) = self.indicators.borrow().as_ref() {
                let (_, indicators_size) = indicators.preferred_size();
                let old = self
                    .indicators_size
                    .replace(Some((indicators_size.width(), indicators_size.height())));

                if let Some(old_indicators_size) = old {
                    if indicators_size.width() != old_indicators_size.0
                        || indicators_size.height() != old_indicators_size.1
                    {
                        obj.update_label_attributes(&indicators_size);
                    }
                } else {
                    obj.update_label_attributes(&indicators_size);
                }

                let (mut minimum, mut natural, minimum_baseline, natural_baseline) =
                    self.label.measure(orientation, for_size);
                let (indicators_min, indicators_nat, _, _) =
                    indicators.measure(orientation, for_size);

                minimum = minimum.max(indicators_min);
                natural = natural.max(indicators_nat);

                if orientation == gtk::Orientation::Vertical && obj.is_opposite_text_direction() {
                    minimum += indicators_min;
                    natural += indicators_nat;
                }

                (minimum, natural, minimum_baseline, natural_baseline)
            } else {
                self.label.measure(orientation, for_size)
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.label.allocate(width, height, baseline, None);
            if let Some(indicators) = self.indicators.borrow().as_ref() {
                indicators.allocate(width, height, baseline, None);
            }
        }

        fn request_mode(&self) -> gtk::SizeRequestMode {
            self.label.request_mode()
        }

        fn direction_changed(&self, _previous_direction: gtk::TextDirection) {
            self.obj().update_label();
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageLabel(ObjectSubclass<imp::MessageLabel>)
        @extends gtk::Widget;
}

impl MessageLabel {
    fn update_label_attributes(&self, indicators_size: &gtk::Requisition) {
        let imp = self.imp();
        if let Some(start_index) = imp.label.text().find(OBJECT_REPLACEMENT_CHARACTER) {
            let attrs = pango::AttrList::new();
            let width = indicators_size.width() + INDICATORS_SPACING;
            let height = indicators_size.height();
            let logical_rect = pango::Rectangle::new(
                0,
                -(height - (height / 4)) * pango::SCALE,
                width * pango::SCALE,
                height * pango::SCALE,
            );
            let mut shape = pango::AttrShape::new(&logical_rect, &logical_rect);

            shape.set_start_index(start_index as u32);
            shape.set_end_index((start_index + OBJECT_REPLACEMENT_CHARACTER.len_utf8()) as u32);
            attrs.insert(shape);

            imp.label.set_attributes(Some(&attrs));
        } else {
            imp.label.set_attributes(None);
        }
    }

    fn is_opposite_text_direction(&self) -> bool {
        let text = self.imp().text.borrow();
        let text_direction = pango::find_base_dir(&text);
        let widget_direction = self.direction();

        (text_direction == pango::Direction::Rtl && widget_direction == gtk::TextDirection::Ltr)
            || text_direction == pango::Direction::Ltr
                && widget_direction == gtk::TextDirection::Rtl
    }

    fn update_label(&self) {
        let imp = self.imp();
        let text = imp.text.borrow();
        if let Some(indicators) = imp.indicators.borrow().as_ref() {
            if !self.is_opposite_text_direction() {
                imp.label
                    .set_label(&format!("{text}{OBJECT_REPLACEMENT_CHARACTER}"));
            } else {
                imp.label.set_label(&text);
            }

            let (_, indicators_size) = indicators.preferred_size();
            self.update_label_attributes(&indicators_size);
        } else {
            imp.label.set_label(&text);
        }
    }

    pub(crate) fn label(&self) -> String {
        self.imp().text.borrow().clone()
    }

    pub(crate) fn set_label(&self, label: String) {
        let imp = self.imp();
        let old = imp.text.replace(label);
        if old != *imp.text.borrow() {
            self.update_label();
            self.notify("label");
        }
    }

    pub(crate) fn add_label_class(&self, class: &str) {
        self.imp().label.add_css_class(class);
    }

    pub(crate) fn indicators(&self) -> Option<ui::MessageIndicators> {
        self.imp().indicators.borrow().clone()
    }

    pub(crate) fn set_indicators(&self, indicators: Option<ui::MessageIndicators>) {
        let imp = self.imp();
        let old = imp.indicators.replace(indicators);
        if old != *imp.indicators.borrow() {
            if let Some(old) = old {
                old.unparent();
            }

            if let Some(indicators) = imp.indicators.borrow().as_ref() {
                indicators.set_parent(self);
            }

            self.update_label();
            self.notify("indicators");
        }
    }
}
