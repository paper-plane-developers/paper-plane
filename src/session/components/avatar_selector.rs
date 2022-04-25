use ashpd::desktop::camera::CameraProxy;
use ashpd::desktop::file_chooser::{FileChooserProxy, FileFilter, OpenFileOptions};
use ashpd::{zbus, WindowIdentifier};
use gettextrs::gettext;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use gtk::{gio, prelude::*};

use crate::take_picture_dialog::TakePictureDialog;

const PHOTO_MIME_TYPES: &[&str] = &["image/png", "image/jpeg"];

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::{gio, CompositeTemplate};
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use crate::utils::spawn;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/components-avatar-selector.ui")]
    pub(crate) struct AvatarSelector {
        pub(super) user_name: RefCell<String>,
        #[template_child]
        pub(super) avatar: TemplateChild<adw::Avatar>,
        #[template_child]
        pub(super) popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub(super) flow_box: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AvatarSelector {
        const NAME: &'static str = "ComponentsAvatarSelector";
        type Type = super::AvatarSelector;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("avatar.choose-file", None, |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    widget.select_file().await;
                }));
            });
            klass.install_action("avatar.take-picture", None, |widget, _, _| {
                widget.start_camera();
            });
            klass.install_action("avatar.reset", None, |widget, _, _| {
                widget.reset();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AvatarSelector {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "user-name",
                        "User Name",
                        "The user name",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "image",
                        "Image",
                        "The image",
                        gdk::Paintable::static_type(),
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
                "user-name" => obj.set_user_name(value.get().unwrap_or_default()),
                "image" => obj.set_image(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "user-name" => obj.user_name().to_value(),
                "image" => obj.image().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.avatar
                .connect_custom_image_notify(clone!(@weak obj => move |_| obj.notify("image")));

            let faces_base_path = "/com/github/melix99/telegrand/faces";
            let faces = gio::ListStore::new(gdk::Texture::static_type());
            gio::resources_enumerate_children(faces_base_path, gio::ResourceLookupFlags::NONE)
                .unwrap()
                .iter()
                .for_each(|face| {
                    let texture = gdk::Texture::from_resource(&format!("{faces_base_path}/{face}"));
                    faces.append(&texture);
                });

            self.flow_box
                .connect_child_activated(clone!(@weak obj => move |_, child| {
                    let paintable = child
                        .child()
                        .unwrap()
                        .downcast_ref::<adw::Avatar>()
                        .unwrap()
                        .custom_image();

                    let imp = obj.imp();
                    imp.avatar.set_custom_image(paintable.as_ref());
                    imp.popover.popdown();
                }));

            self.flow_box.bind_model(Some(&faces), |face| {
                adw::Avatar::builder()
                    .size(80)
                    .custom_image(face.downcast_ref::<gdk::Paintable>().unwrap())
                    .build()
                    .upcast()
            });

            obj.action_set_enabled("avatar.take-picture", false);
            spawn(clone!(@weak obj => async move {
                obj.action_set_enabled("avatar.take-picture", camera_available().await.unwrap_or_default());
            }));
        }
    }

    impl WidgetImpl for AvatarSelector {}
    impl BinImpl for AvatarSelector {}
}

glib::wrapper! {
    pub(crate) struct AvatarSelector(ObjectSubclass<imp::AvatarSelector>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AvatarSelector {
    pub(crate) fn user_name(&self) -> String {
        self.imp().user_name.borrow().to_owned()
    }

    pub(crate) fn set_user_name(&self, user_name: String) {
        if self.user_name() == user_name {
            return;
        }
        self.imp().avatar.set_text(Some(&user_name));
        self.imp().user_name.replace(user_name);
        self.notify("user-name");
    }

    pub(crate) fn image(&self) -> Option<gdk::Paintable> {
        self.imp().avatar.custom_image()
    }

    pub(crate) fn set_image(&self, image: Option<&gdk::Paintable>) {
        self.imp().avatar.set_custom_image(image);
    }

    fn start_camera(&self) {
        let parent_window = self.root().unwrap().downcast().ok();
        TakePictureDialog::new(&parent_window).present();
        self.imp().popover.popdown();
    }

    async fn select_file(&self) {
        let connection = zbus::Connection::session().await.unwrap();
        let proxy = FileChooserProxy::new(&connection).await.unwrap();
        let native = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&native).await;
        let mut filter = FileFilter::new(&gettext("Image"));

        for mime in PHOTO_MIME_TYPES {
            filter = filter.mimetype(mime);
        }

        let options = OpenFileOptions::default().modal(true).add_filter(filter);

        if let Ok(files) = proxy
            .open_file(&identifier, &gettext("Select Avatar"), options)
            .await
        {
            // let parent_window = self.root().unwrap().downcast().ok();
            let file = gio::File::for_uri(&files.uris()[0]);

            if let Some(path) = file.path() {
                let path = path.to_str().unwrap().to_string();
                println!("{path:?}");
                // SendPhotoDialog::new(&parent_window, chat, path).present();
            }
        }
    }

    pub(crate) fn reset(&self) {
        let imp = self.imp();
        imp.avatar.set_custom_image(gdk::Paintable::NONE);
        imp.avatar.set_text(Some(""));
        imp.avatar.set_text(Some(self.user_name().as_str()));
        imp.popover.popdown();
    }
}

async fn camera_available() -> ashpd::Result<bool> {
    let connection = zbus::Connection::session().await?;
    let proxy = CameraProxy::new(&connection).await?;
    proxy.is_camera_present().await
}
