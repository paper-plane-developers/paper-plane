use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};

use crate::session::content::message_row::MediaPicture;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-media.ui")]
    pub(crate) struct Media {
        #[template_child]
        pub(super) overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub(super) picture: TemplateChild<MediaPicture>,
        #[template_child]
        pub(super) progress_bar: TemplateChild<gtk::ProgressBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Media {
        const NAME: &'static str = "ContentMessageMedia";
        type Type = super::Media;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_css_name("messagemedia");
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Media {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecDouble::new(
                    "download-progress",
                    "Download Progress",
                    "The download progress",
                    0.0,
                    1.0,
                    0.0,
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
                "download-progress" => obj.set_download_progress(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "download-progress" => obj.download_progress().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.overlay.unparent();
        }
    }

    impl WidgetImpl for Media {}
}

glib::wrapper! {
    pub(crate) struct Media(ObjectSubclass<imp::Media>)
        @extends gtk::Widget;
}

impl Default for Media {
    fn default() -> Self {
        Self::new()
    }
}

impl Media {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Media")
    }

    pub(crate) fn set_aspect_ratio(&self, aspect_ratio: f64) {
        self.imp().picture.set_aspect_ratio(aspect_ratio);
    }

    pub(crate) fn set_paintable(&self, paintable: Option<gdk::Paintable>) {
        self.imp().picture.set_paintable(paintable);
    }

    pub(crate) fn download_progress(&self) -> f64 {
        self.imp().progress_bar.fraction()
    }

    pub(crate) fn set_download_progress(&self, progress: f64) {
        if self.download_progress() == progress {
            return;
        }

        let imp = self.imp();
        imp.progress_bar.set_fraction(progress);
        imp.progress_bar.set_visible(progress < 1.0);

        self.notify("download-progress");
    }
}
