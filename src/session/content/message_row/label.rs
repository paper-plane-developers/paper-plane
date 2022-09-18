use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango, CompositeTemplate};

use crate::session::content::message_row::MessageIndicators;

const OBJECT_REPLACEMENT_CHARACTER: char = '\u{FFFC}';
const INDICATORS_SPACING: i32 = 6;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="MessageLabel" parent="GtkWidget">
        <child>
          <object class="GtkLabel" id="label">
            <property name="use-markup">True</property>
            <property name="wrap">True</property>
            <property name="wrap-mode">word-char</property>
            <property name="xalign">0</property>
            <property name="yalign">0</property>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    pub(crate) struct MessageLabel {
        pub(super) text: RefCell<String>,
        pub(super) indicators: RefCell<Option<MessageIndicators>>,
        pub(super) indicators_size: RefCell<Option<(i32, i32)>>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageLabel {
        const NAME: &'static str = "MessageLabel";
        type Type = super::MessageLabel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_css_name("messagelabel");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageLabel {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "label",
                        "Label",
                        "The label of the widget",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "indicators",
                        "Indicators",
                        "The message indicators of the widget",
                        MessageIndicators::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "label" => obj.set_label(value.get().unwrap()),
                "indicators" => obj.set_indicators(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "label" => obj.label().to_value(),
                "indicators" => obj.indicators().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.label.unparent();
            if let Some(indicators) = self.indicators.take() {
                indicators.unparent();
            }
        }
    }

    impl WidgetImpl for MessageLabel {
        fn measure(
            &self,
            widget: &Self::Type,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            if let Some(indicators) = self.indicators.borrow().as_ref() {
                let (_, indicators_size) = indicators.preferred_size();
                let old = self
                    .indicators_size
                    .replace(Some((indicators_size.width(), indicators_size.height())));

                if let Some(old_indicators_size) = old {
                    if indicators_size.width() != old_indicators_size.0
                        || indicators_size.height() != old_indicators_size.1
                    {
                        widget.update_label_attributes(&indicators_size);
                    }
                } else {
                    widget.update_label_attributes(&indicators_size);
                }

                let (mut minimum, mut natural, minimum_baseline, natural_baseline) =
                    self.label.measure(orientation, for_size);
                let (indicators_min, indicators_nat, _, _) =
                    indicators.measure(orientation, for_size);

                minimum = minimum.max(indicators_min);
                natural = natural.max(indicators_nat);

                if orientation == gtk::Orientation::Vertical && widget.is_opposite_text_direction()
                {
                    minimum += indicators_min;
                    natural += indicators_nat;
                }

                (minimum, natural, minimum_baseline, natural_baseline)
            } else {
                self.label.measure(orientation, for_size)
            }
        }

        fn size_allocate(&self, _widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.label.allocate(width, height, baseline, None);
            if let Some(indicators) = self.indicators.borrow().as_ref() {
                indicators.allocate(width, height, baseline, None);
            }
        }

        fn request_mode(&self, _widget: &Self::Type) -> gtk::SizeRequestMode {
            self.label.request_mode()
        }

        fn direction_changed(&self, widget: &Self::Type, _previous_direction: gtk::TextDirection) {
            widget.update_label();
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageLabel(ObjectSubclass<imp::MessageLabel>)
        @extends gtk::Widget;
}

impl MessageLabel {
    pub(crate) fn new(label: &str, indicators: Option<&MessageIndicators>) -> Self {
        glib::Object::new(&[("label", &label), ("indicators", &indicators)])
            .expect("Failed to create MessageLabel")
    }

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
                    .set_label(&format!("{}{}", text, OBJECT_REPLACEMENT_CHARACTER));
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

    pub(crate) fn indicators(&self) -> Option<MessageIndicators> {
        self.imp().indicators.borrow().clone()
    }

    pub(crate) fn set_indicators(&self, indicators: Option<MessageIndicators>) {
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
