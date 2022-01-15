use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};

const MIN_WIDTH: i32 = 100;
const MIN_HEIGHT: i32 = 100;
const MAX_HEIGHT: i32 = 400;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct MediaPicture {
        pub paintable: RefCell<Option<gdk::Paintable>>,
        pub aspect_ratio: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MediaPicture {
        const NAME: &'static str = "ContentMediaPicture";
        type Type = super::MediaPicture;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for MediaPicture {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "paintable",
                        "Paintable",
                        "The paintable of the media",
                        gdk::Paintable::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecDouble::new(
                        "aspect-ratio",
                        "Aspect Ratio",
                        "The aspect ratio of the media",
                        0.0,
                        f64::MAX,
                        0.0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "paintable" => obj.set_paintable(value.get().unwrap()),
                "aspect-ratio" => obj.set_aspect_ratio(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "paintable" => obj.paintable().to_value(),
                "aspect-ratio" => obj.aspect_ratio().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MediaPicture {
        fn measure(
            &self,
            _widget: &Self::Type,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            if let gtk::Orientation::Horizontal = orientation {
                let natural = if for_size < 0 {
                    (MAX_HEIGHT as f64 * self.aspect_ratio.get()) as i32
                } else {
                    let for_size = for_size.min(MAX_HEIGHT);
                    (for_size as f64 * self.aspect_ratio.get()) as i32
                }
                .max(MIN_WIDTH);

                (MIN_WIDTH, natural, -1, -1)
            } else {
                let natural = if for_size < 0 {
                    MAX_HEIGHT
                } else {
                    let natural = (for_size as f64 / self.aspect_ratio.get()) as i32;
                    natural.max(MIN_HEIGHT).min(MAX_HEIGHT)
                };

                (MIN_HEIGHT, natural, -1, -1)
            }
        }

        fn request_mode(&self, _widget: &Self::Type) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
            if let Some(paintable) = self.paintable.borrow().as_ref() {
                let widget_width = widget.width() as f64;
                let widget_height = widget.height() as f64;
                let widget_ratio = widget_width / widget_height;
                let paintable_ratio = self.aspect_ratio.get();

                let (x, y, width, height) = if widget_ratio > paintable_ratio {
                    let paintable_height = widget_height / paintable_ratio * widget_ratio;
                    (
                        0.0,
                        (widget_height - paintable_height) / 2.0,
                        widget_width,
                        paintable_height,
                    )
                } else {
                    let paintable_width = widget_width * paintable_ratio / widget_ratio;
                    (
                        (widget_width - paintable_width) / 2.0,
                        0.0,
                        paintable_width,
                        widget_height,
                    )
                };

                snapshot.translate(&graphene::Point::new(x as f32, y as f32));
                paintable.snapshot(snapshot.upcast_ref(), width, height);
            }
        }
    }
}

glib::wrapper! {
    pub struct MediaPicture(ObjectSubclass<imp::MediaPicture>)
        @extends gtk::Widget;
}

impl Default for MediaPicture {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaPicture {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ContentMediaPicture")
    }

    pub fn paintable(&self) -> Option<gdk::Paintable> {
        let self_ = imp::MediaPicture::from_instance(self);
        self_.paintable.borrow().to_owned()
    }

    pub fn set_paintable(&self, paintable: Option<gdk::Paintable>) {
        if self.paintable() == paintable {
            return;
        }

        let self_ = imp::MediaPicture::from_instance(self);
        self_.paintable.replace(paintable);

        self.queue_draw();

        self.notify("paintable");
    }

    pub fn aspect_ratio(&self) -> f64 {
        let self_ = imp::MediaPicture::from_instance(self);
        self_.aspect_ratio.get()
    }

    pub fn set_aspect_ratio(&self, aspect_ratio: f64) {
        if self.aspect_ratio() == aspect_ratio {
            return;
        }

        let self_ = imp::MediaPicture::from_instance(self);
        self_.aspect_ratio.set(aspect_ratio);

        self.queue_resize();

        self.notify("aspect-ratio");
    }
}
