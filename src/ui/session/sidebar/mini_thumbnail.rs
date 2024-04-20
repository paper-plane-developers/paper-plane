use std::cell::RefCell;
use std::sync::OnceLock;

use gtk::gdk;
use gtk::glib;
use gtk::graphene;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct MiniThumbnail(pub(super) RefCell<Option<gdk::Paintable>>);

    #[glib::object_subclass]
    impl ObjectSubclass for MiniThumbnail {
        const NAME: &'static str = "PaplSidebarMiniThumbnail";
        type Type = super::MiniThumbnail;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("minithumbnail");
        }
    }

    impl ObjectImpl for MiniThumbnail {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<gdk::Paintable>("paintable")
                        .explicit_notify()
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "paintable" => obj.set_paintable(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "paintable" => obj.paintable().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_overflow(gtk::Overflow::Hidden);
            obj.set_height_request(16);
            obj.set_width_request(16);
        }
    }

    impl WidgetImpl for MiniThumbnail {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            if let Some(paintable) = obj.paintable() {
                let widget_width = obj.width() as f64;
                let widget_height = obj.height() as f64;
                let widget_ratio = widget_width / widget_height;
                let paintable_ratio = paintable.intrinsic_aspect_ratio();

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
    pub(crate) struct MiniThumbnail(ObjectSubclass<imp::MiniThumbnail>)
        @extends gtk::Widget;
}

impl Default for MiniThumbnail {
    fn default() -> Self {
        Self::new()
    }
}

impl MiniThumbnail {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn paintable(&self) -> Option<gdk::Paintable> {
        self.imp().0.borrow().to_owned()
    }

    pub(crate) fn set_paintable(&self, paintable: Option<gdk::Paintable>) {
        if self.paintable() == paintable {
            return;
        }

        self.imp().0.replace(paintable);
        self.queue_draw();

        self.notify("paintable");
    }
}
