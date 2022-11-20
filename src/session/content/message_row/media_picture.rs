use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};

const MIN_WIDTH: i32 = 100;
const MIN_HEIGHT: i32 = 100;
const MAX_HEIGHT: i32 = 400;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct MediaPicture {
        pub(super) paintable: RefCell<Option<gdk::Paintable>>,
        pub(super) aspect_ratio: Cell<f64>,
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
                    glib::ParamSpecObject::builder::<gdk::Paintable>("paintable")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecDouble::builder("aspect-ratio")
                        .explicit_notify()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "paintable" => obj.set_paintable(value.get().unwrap()),
                "aspect-ratio" => obj.set_aspect_ratio(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "paintable" => obj.paintable().to_value(),
                "aspect-ratio" => obj.aspect_ratio().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MediaPicture {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
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

        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            if let Some(paintable) = self.paintable.borrow().as_ref() {
                let widget_width = obj.width() as f64;
                let widget_height = obj.height() as f64;
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
                paintable.snapshot(snapshot, width, height);
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MediaPicture(ObjectSubclass<imp::MediaPicture>)
        @extends gtk::Widget;
}

impl Default for MediaPicture {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaPicture {
    pub(crate) fn new() -> Self {
        glib::Object::builder().build()
    }

    pub(crate) fn paintable(&self) -> Option<gdk::Paintable> {
        self.imp().paintable.borrow().to_owned()
    }

    pub(crate) fn set_paintable(&self, paintable: Option<gdk::Paintable>) {
        if self.paintable() == paintable {
            return;
        }

        self.imp().paintable.replace(paintable);
        self.queue_draw();

        self.notify("paintable");
    }

    pub(crate) fn aspect_ratio(&self) -> f64 {
        self.imp().aspect_ratio.get()
    }

    pub(crate) fn set_aspect_ratio(&self, aspect_ratio: f64) {
        if self.aspect_ratio() == aspect_ratio {
            return;
        }

        self.imp().aspect_ratio.set(aspect_ratio);
        self.queue_resize();

        self.notify("aspect-ratio");
    }
}
