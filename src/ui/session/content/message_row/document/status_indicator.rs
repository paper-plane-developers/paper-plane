use std::cell::Cell;

use gtk::glib;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use super::file_status::FileStatus;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::StatusIndicator)]
    #[template(
        resource = "/app/drey/paper-plane/ui/session/content/message_row/document/status_indicator.ui"
    )]
    pub(crate) struct StatusIndicator {
        #[property(get, set = Self::set_masked, explicit_notify)]
        pub(super) masked: Cell<bool>,
        #[template_child]
        pub(super) status_image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StatusIndicator {
        const NAME: &'static str = "PaplMessageDocumentStatusIndicator";
        type Type = super::StatusIndicator;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("statusindicator");
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StatusIndicator {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for StatusIndicator {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            if self.masked.get() {
                let widget = self.obj();

                snapshot.push_mask(gsk::MaskMode::InvertedAlpha);

                self.parent_snapshot(snapshot);

                snapshot.pop();

                snapshot.append_color(
                    &widget.color(),
                    &gtk::graphene::Rect::new(
                        0.0,
                        0.0,
                        widget.width() as f32,
                        widget.height() as f32,
                    ),
                );

                snapshot.pop();
            } else {
                self.parent_snapshot(snapshot);
            }
        }
    }

    impl StatusIndicator {
        fn set_masked(&self, masked: bool) {
            if self.masked.replace(masked) != masked {
                let obj = self.obj();
                obj.queue_draw();
                obj.notify_masked();
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct StatusIndicator(ObjectSubclass<imp::StatusIndicator>)
        @extends gtk::Widget;
}

impl StatusIndicator {
    pub(crate) fn set_status(&self, status: FileStatus) {
        let icon_name = match status {
            FileStatus::Downloading(_) | FileStatus::Uploading(_) => "media-playback-stop-symbolic",
            FileStatus::CanBeDownloaded => "document-save-symbolic",
            FileStatus::Downloaded => "folder-documents-symbolic",
        };

        self.imp().status_image.set_icon_name(Some(icon_name));
    }
}
