use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::{gio, glib};

use tdlib::enums::StickerFormat;
use tdlib::types::Sticker as TdSticker;

use super::VectorPath;
use crate::session::Session;
use crate::utils::{decode_image_from_path, spawn};

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::Sticker)]
    pub(crate) struct Sticker {
        pub(super) file_id: Cell<i32>,
        pub(super) aspect_ratio: Cell<f64>,
        pub(super) child: RefCell<Option<gtk::Widget>>,

        #[property(get, set = Self::set_longer_side_size)]
        pub(super) longer_side_size: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sticker {
        const NAME: &'static str = "ComponentsSticker";
        type Type = super::Sticker;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for Sticker {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn dispose(&self) {
            if let Some(child) = self.child.replace(None) {
                child.unparent()
            }
        }
    }

    impl WidgetImpl for Sticker {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let size = self.longer_side_size.get();
            let aspect_ratio = self.aspect_ratio.get();

            let min_size = if let Some(child) = &*self.child.borrow_mut() {
                child.measure(orientation, for_size).0
            } else {
                1
            };

            let size = if let gtk::Orientation::Horizontal = orientation {
                if aspect_ratio >= 1.0 {
                    size
                } else {
                    (size as f64 * aspect_ratio) as i32
                }
            } else if aspect_ratio >= 1.0 {
                (size as f64 / aspect_ratio) as i32
            } else {
                size
            }
            .max(min_size);

            (size, size, -1, -1)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(child) = &*self.child.borrow() {
                child.allocate(width, height, baseline, None);
            }
        }
    }

    impl Sticker {
        fn set_longer_side_size(&self, size: i32) {
            self.longer_side_size.set(size);
            self.obj().queue_resize();
        }
    }
}

glib::wrapper! {
    pub(crate) struct Sticker(ObjectSubclass<imp::Sticker>)
        @extends gtk::Widget;
}

impl Sticker {
    pub(crate) fn update_sticker(&self, sticker: TdSticker, looped: bool, session: Session) {
        let imp = self.imp();

        let file_id = sticker.sticker.id;
        if self.imp().file_id.replace(file_id) == file_id {
            return;
        }

        self.set_child(Some(VectorPath::new(&sticker.outline).upcast()));

        let aspect_ratio = sticker.width as f64 / sticker.height as f64;
        imp.aspect_ratio.set(aspect_ratio);

        let format = sticker.format;

        spawn(clone!(@weak self as obj, @weak session => async move {
                if sticker.sticker.local.is_downloading_completed {
                    obj.load_sticker(&sticker.sticker.local.path, file_id, looped, format).await;
                } else {
                obj.download_sticker(file_id, &session, looped, format).await
            }
        }));
    }

    pub(crate) fn play_animation(&self) {
        if let Some(animation) = &*self.imp().child.borrow() {
            if let Some(animation) = animation.downcast_ref::<rlt::Animation>() {
                if !animation.is_playing() {
                    animation.play();
                }
            }
        }
    }

    async fn download_sticker(
        &self,
        file_id: i32,
        session: &Session,
        looped: bool,
        format: StickerFormat,
    ) {
        match session.download_file(file_id).await {
            Ok(file) => {
                self.load_sticker(&file.local.path, file_id, looped, format)
                    .await;
            }
            Err(e) => {
                log::warn!("Failed to download a sticker: {e:?}");
            }
        }
    }

    async fn load_sticker(&self, path: &str, file_id: i32, looped: bool, format: StickerFormat) {
        let path = path.to_owned();

        let widget: gtk::Widget = match format {
            StickerFormat::Tgs => {
                let animation = rlt::Animation::from_filename(&path);
                animation.set_loop(looped);
                animation.use_cache(looped);
                animation.play();
                animation.upcast()
            }
            StickerFormat::Webp => {
                let result = gio::spawn_blocking(move || decode_image_from_path(&path))
                    .await
                    .unwrap();

                match result {
                    Ok(texture) => {
                        let picture = gtk::Picture::new();
                        picture.set_paintable(Some(&texture));
                        picture.upcast()
                    }
                    Err(e) => {
                        log::warn!("Error decoding a sticker: {e:?}");
                        return;
                    }
                }
            }
            _ => unimplemented!(),
        };

        // Skip if widget was recycled by ListView
        if self.imp().file_id.get() == file_id {
            self.set_child(Some(widget));
        }
    }

    fn set_child(&self, child: Option<gtk::Widget>) {
        let imp = self.imp();

        if let Some(ref child) = child {
            child.set_parent(self);
        }

        if let Some(old) = imp.child.replace(child) {
            old.unparent()
        }
    }
}
