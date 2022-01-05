use gtk::{gdk, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::session::content::message_row::MediaPicture;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-media.ui")]
    pub struct Media {
        #[template_child]
        pub content: TemplateChild<gtk::Box>,
        #[template_child]
        pub picture: TemplateChild<MediaPicture>,
        #[template_child]
        pub caption_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Media {
        const NAME: &'static str = "ContentMessageMedia";
        type Type = super::Media;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Media {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "caption",
                        "Caption",
                        "The caption",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_double(
                        "download-progress",
                        "Download Progress",
                        "The download progress",
                        0.0,
                        1.0,
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
                "caption" => obj.set_caption(value.get().unwrap()),
                "download-progress" => obj.set_download_progress(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "caption" => obj.caption().to_value(),
                "download-progress" => obj.download_progress().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.content.unparent();
        }
    }

    impl WidgetImpl for Media {
        fn measure(
            &self,
            _widget: &Self::Type,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            if let gtk::Orientation::Horizontal = orientation {
                let (mut minimum, mut natural, minimum_baseline, natural_baseline) =
                    self.content.measure(orientation, for_size);

                if for_size == -1 {
                    let (_, media_default_width, _, _) =
                        self.picture.measure(gtk::Orientation::Horizontal, -1);
                    minimum = minimum.min(media_default_width);
                    natural = media_default_width;
                }

                (minimum, natural, minimum_baseline, natural_baseline)
            } else {
                let for_size = if for_size == -1 {
                    let (_, media_default_width, _, _) =
                        self.picture.measure(gtk::Orientation::Horizontal, -1);
                    media_default_width
                } else {
                    for_size
                };

                self.content.measure(orientation, for_size)
            }
        }

        fn size_allocate(&self, _widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.content.allocate(width, height, baseline, None);
        }

        fn request_mode(&self, _widget: &Self::Type) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }
    }
}

glib::wrapper! {
    pub struct Media(ObjectSubclass<imp::Media>)
        @extends gtk::Widget;
}

impl Default for Media {
    fn default() -> Self {
        Self::new()
    }
}

impl Media {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Media")
    }

    pub fn set_aspect_ratio(&self, aspect_ratio: f64) {
        let self_ = imp::Media::from_instance(self);
        self_.picture.set_aspect_ratio(aspect_ratio);
    }

    pub fn set_paintable(&self, paintable: Option<gdk::Paintable>) {
        let self_ = imp::Media::from_instance(self);
        self_.picture.set_paintable(paintable);
    }

    pub fn caption(&self) -> String {
        let self_ = imp::Media::from_instance(self);
        self_.caption_label.label().into()
    }

    pub fn set_caption(&self, caption: &str) {
        if self.caption() == caption {
            return;
        }

        let self_ = imp::Media::from_instance(self);
        if caption.is_empty() {
            self_.caption_label.set_visible(false);
            self.remove_css_class("with-caption");
        } else {
            self_.caption_label.set_visible(true);
            self.add_css_class("with-caption");
        }

        self_.caption_label.set_label(caption);
        self.notify("caption");
    }

    pub fn download_progress(&self) -> f64 {
        let self_ = imp::Media::from_instance(self);
        self_.progress_bar.fraction()
    }

    pub fn set_download_progress(&self, progress: f64) {
        if self.download_progress() == progress {
            return;
        }

        let self_ = imp::Media::from_instance(self);
        self_.progress_bar.set_fraction(progress);
        self_.progress_bar.set_visible(progress < 1.0);

        self.notify("download-progress");
    }
}
