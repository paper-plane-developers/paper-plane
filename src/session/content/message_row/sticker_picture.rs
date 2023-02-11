use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    const SIZE: i32 = 176;

    #[derive(Debug, Default)]
    pub(crate) struct StickerPicture {
        pub(super) texture: RefCell<Option<gdk::Texture>>,
        pub(super) aspect_ratio: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StickerPicture {
        const NAME: &'static str = "MessageStickerPicture";
        type Type = super::StickerPicture;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for StickerPicture {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<gdk::Texture>("texture")
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
                "texture" => obj.set_texture(value.get().unwrap()),
                "aspect-ratio" => obj.set_aspect_ratio(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "texture" => obj.texture().to_value(),
                "aspect-ratio" => obj.aspect_ratio().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for StickerPicture {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let aspect_ratio = self.aspect_ratio.get();
            let size = if let gtk::Orientation::Horizontal = orientation {
                if aspect_ratio >= 1.0 {
                    SIZE
                } else {
                    (SIZE as f64 * aspect_ratio) as i32
                }
            } else if aspect_ratio >= 1.0 {
                (SIZE as f64 / aspect_ratio) as i32
            } else {
                SIZE
            };

            (size, size, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            if let Some(texture) = self.texture.borrow().as_ref() {
                let width = obj.width() as f64;
                let height = obj.height() as f64;
                texture.snapshot(snapshot, width, height);
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct StickerPicture(ObjectSubclass<imp::StickerPicture>)
        @extends gtk::Widget;
}

impl Default for StickerPicture {
    fn default() -> Self {
        Self::new()
    }
}

impl StickerPicture {
    pub(crate) fn new() -> Self {
        glib::Object::builder().build()
    }

    pub(crate) fn aspect_ratio(&self) -> f64 {
        self.imp().aspect_ratio.get()
    }

    pub(crate) fn set_aspect_ratio(&self, aspect_ratio: f64) {
        if self.aspect_ratio() == aspect_ratio {
            return;
        }

        self.imp().aspect_ratio.replace(aspect_ratio);
        self.queue_resize();

        self.notify("aspect-ratio");
    }

    pub(crate) fn texture(&self) -> Option<gdk::Texture> {
        self.imp().texture.borrow().to_owned()
    }

    pub(crate) fn set_texture(&self, texture: Option<gdk::Texture>) {
        if self.texture() == texture {
            return;
        }

        self.imp().texture.replace(texture);
        self.queue_draw();

        self.notify("texture");
    }
}
