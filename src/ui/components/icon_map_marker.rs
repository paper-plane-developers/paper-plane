use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/components/icon_map_marker.ui")]
    pub(crate) struct AvatarMapMarker {
        #[template_child]
        pub(super) icon: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AvatarMapMarker {
        const NAME: &'static str = "PaplIconMapMarker";
        type Type = super::IconMapMarker;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            klass.set_css_name("iconmapmarker");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AvatarMapMarker {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecString::builder("icon-name")
                    .construct()
                    .explicit_notify()
                    .build()]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "icon-name" => self.obj().set_icon_name(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "icon-name" => self.obj().icon_name().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            utils::unparent_children(&*self.obj());
        }
    }

    impl WidgetImpl for AvatarMapMarker {}

    #[gtk::template_callbacks]
    impl AvatarMapMarker {
        #[template_callback]
        fn on_icon_notify_icon_name(&self) {
            self.obj().notify("icon-name");
        }
    }
}

glib::wrapper! {
    pub(crate) struct IconMapMarker(ObjectSubclass<imp::AvatarMapMarker>)
        @extends gtk::Widget;
}

impl From<Option<&str>> for IconMapMarker {
    fn from(icon_name: Option<&str>) -> Self {
        glib::Object::builder()
            .property("icon-name", icon_name)
            .build()
    }
}

impl IconMapMarker {
    pub(crate) fn icon_name(&self) -> Option<glib::GString> {
        self.imp().icon.icon_name()
    }

    pub(crate) fn set_icon_name(&self, icon_name: Option<&str>) {
        self.imp().icon.set_icon_name(icon_name);
    }
}
