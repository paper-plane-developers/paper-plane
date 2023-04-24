// Based on LabelWithWidgets from Fractal
// https://gitlab.gnome.org/GNOME/fractal/-/blob/main/src/components/label_with_widgets.rs

use gtk::{glib, glib::clone, pango, prelude::*, subclass::prelude::*};

pub const DEFAULT_PLACEHOLDER: &str = "<widget>";
const OBJECT_REPLACEMENT_CHARACTER: &str = "\u{FFFC}";

mod imp {
    use std::cell::{Cell, RefCell};

    use super::*;

    #[derive(Debug, Default)]
    pub struct LabelWithWidgets {
        pub widgets: RefCell<Vec<gtk::Widget>>,
        pub widgets_sizes: RefCell<Vec<(i32, i32)>>,
        pub label: gtk::Label,
        pub placeholder: RefCell<Option<String>>,
        pub text: RefCell<String>,
        pub ellipsize: Cell<bool>,
        pub reserved_space: Cell<(i32, i32)>,
        pub reserved_space_changed: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LabelWithWidgets {
        const NAME: &'static str = "LabelWithWidgets";
        type Type = super::LabelWithWidgets;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Buildable,);
    }

    impl ObjectImpl for LabelWithWidgets {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("label")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecString::builder("placeholder")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("use-markup")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("ellipsize")
                        .explicit_notify()
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "label" => obj.set_label(value.get().unwrap()),
                "placeholder" => obj.set_placeholder(value.get().unwrap()),
                "use-markup" => obj.set_use_markup(value.get().unwrap()),
                "ellipsize" => obj.set_ellipsize(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "label" => obj.label().to_value(),
                "placeholder" => obj.placeholder().to_value(),
                "use-markup" => obj.uses_markup().to_value(),
                "ellipsize" => obj.ellipsize().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let label = &self.label;
            label.set_parent(&*obj);
            label.set_wrap(true);
            label.set_wrap_mode(pango::WrapMode::WordChar);
            label.set_use_markup(true);
            label.set_xalign(0.0);
            label.set_valign(gtk::Align::Start);
            label.add_css_class("line-height");
            label.connect_notify_local(
                Some("label"),
                clone!(@weak obj => move |_, _| {
                    obj.invalidate_child_widgets();
                }),
            );
        }

        fn dispose(&self) {
            self.label.unparent();
            for widget in self.widgets.borrow().iter() {
                widget.unparent();
            }
        }
    }

    impl WidgetImpl for LabelWithWidgets {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.obj().allocate_shapes();
            self.label.measure(orientation, for_size)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.label.allocate(width, height, baseline, None);
            self.obj().allocate_children();
        }

        fn request_mode(&self) -> gtk::SizeRequestMode {
            self.label.request_mode()
        }

        fn direction_changed(&self, _previous_direction: gtk::TextDirection) {
            self.obj().update_label();
        }
    }

    impl BuildableImpl for LabelWithWidgets {
        fn add_child(&self, builder: &gtk::Builder, child: &glib::Object, type_: Option<&str>) {
            if let Some(child) = child.downcast_ref::<gtk::Widget>() {
                self.obj().append_child(child);
            } else {
                self.parent_add_child(builder, child, type_)
            }
        }
    }
}

glib::wrapper! {
    /// A Label that can have multiple widgets placed inside the text.
    ///
    /// By default the string "<widget>" will be used as location to place the
    /// child widgets. You can set your own placeholder if you need.
    pub struct LabelWithWidgets(ObjectSubclass<imp::LabelWithWidgets>)
        @extends gtk::Widget, @implements gtk::Accessible, gtk::Buildable;
}

impl LabelWithWidgets {
    /// Create an empty `LabelWithWidget`.
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Create a `LabelWithWidget` with the given label and widgets.
    pub fn with_label_and_widgets<P: IsA<gtk::Widget>>(label: &str, widgets: Vec<P>) -> Self {
        let obj: Self = glib::Object::builder().property("label", label).build();
        // FIXME: use a property for widgets
        obj.set_widgets(widgets);
        obj
    }

    pub fn append_child<P: IsA<gtk::Widget>>(&self, child: &P) {
        self.imp().widgets.borrow_mut().push(child.clone().upcast());
        child.set_parent(self);
        self.invalidate_child_widgets();
    }

    pub fn set_widgets<P: IsA<gtk::Widget>>(&self, widgets: Vec<P>) {
        let imp = self.imp();

        for widget in imp.widgets.take() {
            widget.unparent();
        }

        imp.widgets
            .borrow_mut()
            .append(&mut widgets.into_iter().map(|w| w.upcast()).collect());

        for child in imp.widgets.borrow().iter() {
            child.set_parent(self);
        }
        self.invalidate_child_widgets();
    }

    pub fn widgets(&self) -> Vec<gtk::Widget> {
        self.imp().widgets.borrow().to_owned()
    }

    /// Set the text of the label.
    pub fn set_label(&self, mut label: String) {
        let imp = self.imp();

        if imp.text.borrow().as_str() == label.as_str() {
            return;
        }

        label.push_str(OBJECT_REPLACEMENT_CHARACTER);

        imp.text.replace(label);
        self.update_label();
        self.notify("label");
    }

    /// The text of the label.
    pub fn label(&self) -> String {
        self.imp().text.borrow().to_owned()
    }

    /// Set the placeholder that is replaced with widgets.
    pub fn set_placeholder(&self, placeholder: Option<String>) {
        let imp = self.imp();

        if imp.placeholder.borrow().as_ref() == placeholder.as_ref() {
            return;
        }

        imp.placeholder.replace(placeholder);
        self.update_label();
        self.notify("placeholder");
    }

    /// The placeholder that is replaced with widgets.
    ///
    /// Defaults to `<widget>`.
    pub fn placeholder(&self) -> Option<String> {
        self.imp().placeholder.borrow().to_owned()
    }

    fn invalidate_child_widgets(&self) {
        self.imp().widgets_sizes.borrow_mut().clear();
        self.queue_resize();
    }

    fn allocate_shapes(&self) {
        let imp = self.imp();
        let mut widgets_sizes = imp.widgets_sizes.borrow_mut();

        let mut child_size_changed = imp.reserved_space_changed.take();
        for (i, child) in imp.widgets.borrow().iter().enumerate() {
            let (_, natural_size) = child.preferred_size();
            let width = natural_size.width();
            let height = natural_size.height();
            if let Some((old_width, old_height)) = widgets_sizes.get(i) {
                if old_width != &width || old_height != &height {
                    let _ = std::mem::replace(&mut widgets_sizes[i], (width, height));
                    child_size_changed = true;
                }
            } else {
                widgets_sizes.insert(i, (width, height));
                child_size_changed = true;
            }
        }

        if !child_size_changed {
            return;
        }

        let attrs = pango::AttrList::new();
        for (i, (start_index, _)) in imp
            .label
            .text()
            .as_str()
            .match_indices(OBJECT_REPLACEMENT_CHARACTER)
            .enumerate()
        {
            if let Some((width, height)) = widgets_sizes.get(i) {
                let logical_rect = pango::Rectangle::new(
                    0,
                    -(height - (height / 4)) * pango::SCALE,
                    width * pango::SCALE,
                    height * pango::SCALE,
                );

                let mut shape = pango::AttrShape::new(&logical_rect, &logical_rect);
                shape.set_start_index(start_index as u32);
                shape.set_end_index((start_index + OBJECT_REPLACEMENT_CHARACTER.len()) as u32);
                attrs.insert(shape);
            } else {
                {
                    let (width, height) = imp.reserved_space.get();
                    let logical_rect = pango::Rectangle::new(
                        0,
                        -(height - (height / 4)) * pango::SCALE,
                        width * pango::SCALE,
                        height * pango::SCALE,
                    );

                    let mut shape = pango::AttrShape::new(&logical_rect, &logical_rect);
                    shape.set_start_index(start_index as u32);
                    shape.set_end_index((start_index + OBJECT_REPLACEMENT_CHARACTER.len()) as u32);
                    attrs.insert(shape);
                }

                break;
            }
        }

        imp.label.set_attributes(Some(&attrs));
    }

    pub(crate) fn reserve_space(&self, width: i32, height: i32) {
        self.imp().reserved_space.set((width, height));
        self.imp().reserved_space_changed.set(true);
        self.invalidate_child_widgets();
    }

    fn allocate_children(&self) {
        let imp = self.imp();
        let widgets = imp.widgets.borrow();
        let widgets_sizes = imp.widgets_sizes.borrow();

        let mut run_iter = imp.label.layout().iter();
        let mut i = 0;
        loop {
            if let Some(run) = run_iter.run_readonly() {
                if run
                    .item()
                    .analysis()
                    .extra_attrs()
                    .iter()
                    .any(|attr| attr.type_() == pango::AttrType::Shape)
                {
                    if let Some(widget) = widgets.get(i) {
                        let (width, height) = widgets_sizes[i];
                        let (_, mut extents) = run_iter.run_extents();
                        pango::extents_to_pixels(Some(&mut extents), None);

                        let (offset_x, offset_y) = imp.label.layout_offsets();
                        let allocation = gtk::Allocation::new(
                            extents.x() + offset_x,
                            extents.y() + offset_y,
                            width,
                            height,
                        );
                        widget.size_allocate(&allocation, -1);
                        i += 1;
                    } else {
                        break;
                    }
                }
            }
            if !run_iter.next_run() {
                break;
            }
        }
    }

    /// Whether the label's text is interpreted as Pango markup.
    pub fn uses_markup(&self) -> bool {
        self.imp().label.uses_markup()
    }

    /// Sets whether the text of the label contains markup.
    pub fn set_use_markup(&self, use_markup: bool) {
        self.imp().label.set_use_markup(use_markup);
    }

    /// Whether the text of the label is ellipsized.
    pub fn ellipsize(&self) -> bool {
        self.imp().ellipsize.get()
    }

    /// Sets whether the text of the label should be ellipsized.
    pub fn set_ellipsize(&self, ellipsize: bool) {
        if self.ellipsize() == ellipsize {
            return;
        }

        self.imp().ellipsize.set(true);
        self.update_label();
        self.notify("ellipsize");
    }

    fn update_label(&self) {
        let imp = self.imp();
        if self.ellipsize() {
            // Workaround: if both wrap and ellipsize are set, and there are
            // widgets inserted, GtkLabel reports an erroneous minimum width.
            imp.label.set_wrap(false);
            imp.label.set_ellipsize(pango::EllipsizeMode::End);

            let label = &*imp.text.borrow();
            let placeholder = imp.placeholder.borrow();
            let placeholder = placeholder.as_deref().unwrap_or(DEFAULT_PLACEHOLDER);
            let label = label.replace(placeholder, OBJECT_REPLACEMENT_CHARACTER);
            let label = if let Some(pos) = label.find('\n') {
                format!("{}…", &label[0..pos])
            } else {
                label
            };
            imp.label.set_label(&label);
        } else {
            imp.label.set_wrap(true);
            imp.label.set_ellipsize(pango::EllipsizeMode::None);

            let label = &*imp.text.borrow();
            let placeholder = imp.placeholder.borrow();
            let placeholder = placeholder.as_deref().unwrap_or(DEFAULT_PLACEHOLDER);
            let label = label.replace(placeholder, OBJECT_REPLACEMENT_CHARACTER);
            imp.label.set_label(&label);
        }
        self.invalidate_child_widgets();
    }
}
