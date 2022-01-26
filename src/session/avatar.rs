use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use tdlib::types::{ChatPhotoInfo, File, ProfilePhoto};

use crate::Session;

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct Avatar {
        pub(super) image: RefCell<Option<gdk::Paintable>>,
        pub(super) needed: Cell<bool>,
        pub(super) display_name: RefCell<Option<String>>,
        pub(super) session: OnceCell<Session>,

        pub(super) image_file: RefCell<Option<File>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Avatar {
        const NAME: &'static str = "Avatar";
        type Type = super::Avatar;
    }

    impl ObjectImpl for Avatar {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "image",
                        "Image",
                        "The image of this avatar",
                        gdk::Paintable::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "needed",
                        "Needed",
                        "Whether the image needs to be loaded or not",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "display-name",
                        "Display Name",
                        "The display name used for this avatar",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
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
                "needed" => obj.set_needed(value.get().unwrap()),
                "display-name" => obj.set_display_name(value.get().unwrap()),
                "session" => self.session.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "image" => obj.image().to_value(),
                "needed" => obj.needed().to_value(),
                "display-name" => obj.display_name().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Avatar(ObjectSubclass<imp::Avatar>);
}

impl Avatar {
    pub(crate) fn new(session: &Session) -> Self {
        glib::Object::new(&[("session", session)]).expect("Failed to create Avatar")
    }

    pub(crate) fn update_from_chat_photo(&self, chat_photo: Option<ChatPhotoInfo>) {
        let image_file = chat_photo.map(|data| data.small);
        self.set_image_file(image_file);
    }

    pub(crate) fn update_from_user_photo(&self, user_photo: Option<ProfilePhoto>) {
        let image_file = user_photo.map(|data| data.small);
        self.set_image_file(image_file);
    }

    fn load(&self) {
        if !self.needed() {
            return;
        }

        if let Some(file) = &*self.imp().image_file.borrow() {
            if file.local.is_downloading_completed {
                let gfile = gio::File::for_path(&file.local.path);
                let texture = gdk::Texture::from_file(&gfile).unwrap();

                self.set_image(Some(texture.upcast()));
            } else if file.local.can_be_downloaded && !file.local.is_downloading_active {
                let (sender, receiver) =
                    glib::MainContext::sync_channel::<File>(Default::default(), 5);

                receiver.attach(
                    None,
                    clone!(@weak self as obj => @default-return glib::Continue(false), move |file| {
                        obj.set_image_file(Some(file));

                        glib::Continue(true)
                    }),
                );

                self.session().download_file(file.id, sender);
            }
        }
    }

    fn set_image_file(&self, file: Option<File>) {
        let is_some = file.is_some();
        self.imp().image_file.replace(file);

        if is_some {
            self.load();
        } else {
            self.set_image(None);
        }
    }

    pub(crate) fn image(&self) -> Option<gdk::Paintable> {
        self.imp().image.borrow().clone()
    }

    fn set_image(&self, image: Option<gdk::Paintable>) {
        self.imp().image.replace(image);
        self.notify("image");
    }

    pub(crate) fn needed(&self) -> bool {
        self.imp().needed.get()
    }

    pub(crate) fn set_needed(&self, needed: bool) {
        if self.needed() == needed {
            return;
        }

        self.imp().needed.set(needed);

        if needed {
            self.load();
        }

        self.notify("needed");
    }

    pub(crate) fn display_name(&self) -> Option<String> {
        self.imp().display_name.borrow().clone()
    }

    pub(crate) fn set_display_name(&self, display_name: Option<String>) {
        if self.display_name() == display_name {
            return;
        }

        self.imp().display_name.replace(display_name);
        self.notify("display-name");
    }

    pub(crate) fn session(&self) -> &Session {
        self.imp().session.get().unwrap()
    }
}
