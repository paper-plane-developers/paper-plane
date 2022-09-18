use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};

use crate::session::content::message_row::{MediaPicture, MessageIndicators, MessageLabel};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-media.ui")]
    pub(crate) struct Media {
        pub(super) caption_label: RefCell<Option<MessageLabel>>,
        pub(super) indicators: OnceCell<MessageIndicators>,
        #[template_child]
        pub(super) content: TemplateChild<gtk::Box>,
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
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Media {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "caption",
                        "Caption",
                        "The caption",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecDouble::new(
                        "download-progress",
                        "Download Progress",
                        "The download progress",
                        0.0,
                        1.0,
                        0.0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "indicators",
                        "Indicators",
                        "The message indicators of the widget",
                        MessageIndicators::static_type(),
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
                "caption" => obj.set_caption(value.get().unwrap()),
                "download-progress" => obj.set_download_progress(value.get().unwrap()),
                "indicators" => {
                    let indicators: MessageIndicators = value.get().unwrap();

                    // Add the indicators to the picture overlay by default because we don't
                    // expect to have a caption text in the construct phase
                    self.overlay.add_overlay(&indicators);
                    self.indicators.set(indicators).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "caption" => obj.caption().to_value(),
                "download-progress" => obj.download_progress().to_value(),
                "indicators" => obj.indicators().to_value(),
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

    pub(crate) fn caption(&self) -> String {
        self.imp()
            .caption_label
            .borrow()
            .as_ref()
            .map(|c| c.label())
            .unwrap_or_default()
    }

    pub(crate) fn set_caption(&self, caption: String) {
        if self.caption() == caption {
            return;
        }

        let imp = self.imp();
        let indicators = self.indicators();

        if caption.is_empty() {
            // This will also drop the label and consequently unparent the indicators from it
            imp.content.remove(&imp.caption_label.take().unwrap());

            imp.overlay.add_overlay(indicators);

            self.remove_css_class("with-caption");
        } else {
            let mut caption_label_ref = imp.caption_label.borrow_mut();

            if let Some(caption_label) = caption_label_ref.as_ref() {
                caption_label.set_label(caption);
            } else {
                // Unparent the indicators from the picture overlay
                imp.overlay.remove_overlay(indicators);

                let caption_label = MessageLabel::new(&caption, Some(indicators));
                imp.content.append(&caption_label);

                *caption_label_ref = Some(caption_label);

                self.add_css_class("with-caption");
            }
        }

        self.notify("caption");
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

    pub(crate) fn indicators(&self) -> &MessageIndicators {
        self.imp().indicators.get().unwrap()
    }
}
