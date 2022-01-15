use gtk::{gdk, glib, prelude::*, subclass::prelude::*};

const MAX_SIZE: i32 = 200;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct StickerPaintable {
        pub texture: RefCell<Option<gdk::Texture>>,
        pub width: Cell<i32>,
        pub height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StickerPaintable {
        const NAME: &'static str = "ContentStickerPaintable";
        type Type = super::StickerPaintable;
        type Interfaces = (gdk::Paintable,);
    }

    impl ObjectImpl for StickerPaintable {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "texture",
                    "Texture",
                    "The texture of the sticker",
                    gdk::Texture::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                "texture" => obj.set_texture(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "texture" => obj.texture().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl PaintableImpl for StickerPaintable {
        fn intrinsic_width(&self, _paintable: &Self::Type) -> i32 {
            self.width.get()
        }

        fn intrinsic_height(&self, _paintable: &Self::Type) -> i32 {
            self.height.get()
        }

        fn snapshot(
            &self,
            _paintable: &Self::Type,
            snapshot: &gdk::Snapshot,
            width: f64,
            height: f64,
        ) {
            if let Some(texture) = self.texture.borrow().as_ref() {
                texture.snapshot(snapshot, width, height);
            }
        }
    }
}

glib::wrapper! {
    pub struct StickerPaintable(ObjectSubclass<imp::StickerPaintable>) @implements gdk::Paintable;
}

impl Default for StickerPaintable {
    fn default() -> Self {
        Self::new()
    }
}

impl StickerPaintable {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create StickerPaintable")
    }

    pub fn set_aspect_ratio(&self, aspect_ratio: f64) {
        let self_ = imp::StickerPaintable::from_instance(self);

        if aspect_ratio >= 1.0 {
            self_.width.set(MAX_SIZE);
            self_.height.set((MAX_SIZE as f64 / aspect_ratio) as i32);
        } else {
            self_.width.set((MAX_SIZE as f64 * aspect_ratio) as i32);
            self_.height.set(MAX_SIZE);
        }

        self.invalidate_size();
    }

    pub fn texture(&self) -> Option<gdk::Texture> {
        let self_ = imp::StickerPaintable::from_instance(self);
        self_.texture.borrow().to_owned()
    }

    pub fn set_texture(&self, texture: Option<gdk::Texture>) {
        if self.texture() == texture {
            return;
        }

        let self_ = imp::StickerPaintable::from_instance(self);
        self_.texture.replace(texture);

        self.invalidate_contents();

        self.notify("texture");
    }
}
