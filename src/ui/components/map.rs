use std::sync::OnceLock;

use adw::prelude::*;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use shumate::prelude::*;

use crate::ui;
use crate::utils;

// The golden ratio
const PHI: f64 = 1.6180339887;
const MIN_HEIGHT: i32 = 150;
const MAX_HEIGHT: i32 = 225;
const MIN_WIDTH: i32 = (MIN_HEIGHT as f64 * PHI) as i32;
const MAX_WIDTH: i32 = (MAX_HEIGHT as f64 * PHI) as i32;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "LicensePosition")]
pub(crate) enum LicensePosition {
    #[default]
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/components/map.ui")]
    pub(crate) struct Map {
        pub(super) marker: shumate::Marker,
        #[template_child]
        pub(super) marker_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) map: TemplateChild<shumate::Map>,
        #[template_child]
        pub(super) license_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Map {
        const NAME: &'static str = "PaplMap";
        type Type = super::Map;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);

            klass.set_css_name("map");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Map {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecBoolean::builder("interactive")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecEnum::builder::<LicensePosition>("license-position")
                        .explicit_notify()
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "interactive" => self.obj().set_interactive(value.get().unwrap()),
                "license-position" => self.obj().set_license_position(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "interactive" => self.obj().interactive().to_value(),
                "license-position" => self.obj().license_position().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = &*self.obj();

            obj.set_custom_marker(None);

            let map_source = shumate::MapSourceRegistry::with_defaults()
                .by_id(shumate::MAP_SOURCE_OSM_MAPNIK)
                .unwrap();
            self.map.set_map_source(&map_source);

            let viewport = obj.viewport();

            let map_layer = shumate::MapLayer::new(&map_source, &viewport);
            self.map.add_layer(&map_layer);

            let marker_layer = shumate::MarkerLayer::new(&viewport);
            marker_layer.add_marker(&self.marker);
            self.map.add_layer(&marker_layer);
        }

        fn dispose(&self) {
            utils::unparent_children(&*self.obj());
        }
    }

    impl WidgetImpl for Map {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let (min_size, natural_size) = match orientation {
                gtk::Orientation::Horizontal => self.measure(for_size, MIN_WIDTH, MAX_WIDTH),
                _ => self.measure((for_size as f64 / PHI) as i32, MIN_HEIGHT, MAX_HEIGHT),
            };

            (min_size, natural_size, -1, -1)
        }

        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let mut child = self.obj().first_child();
            while let Some(child_) = child {
                child = child_.next_sibling();
                child_.allocate(width, height, baseline, None);
            }
        }
    }

    #[gtk::template_callbacks]
    impl Map {
        fn measure(&self, for_size: i32, min_size: i32, max_size: i32) -> (i32, i32) {
            let natural_size = if for_size == -1 {
                max_size
            } else {
                let mut child = self.obj().first_child();
                while let Some(child_) = child {
                    child = child_.next_sibling();
                    child_.measure(gtk::Orientation::Horizontal, for_size);
                }

                for_size.min(max_size).max(min_size)
            };
            (min_size, natural_size)
        }

        #[template_callback]
        fn on_license_label_notify_align(&self) {
            self.obj().notify("license-position");
        }
    }
}

glib::wrapper! {
    pub(crate) struct Map(ObjectSubclass<imp::Map>)
        @extends gtk::Widget;
}

impl Map {
    pub(crate) fn interactive(&self) -> bool {
        self.imp().map.is_sensitive()
    }

    pub(crate) fn set_interactive(&self, interactive: bool) {
        self.imp().map.set_sensitive(interactive);
    }

    pub(crate) fn license_position(&self) -> LicensePosition {
        let license_label = self.imp().license_label.get();

        if license_label.valign() == gtk::Align::Start {
            if license_label.halign() == gtk::Align::Start {
                LicensePosition::TopLeft
            } else {
                LicensePosition::TopRight
            }
        } else if license_label.valign() == gtk::Align::Start {
            LicensePosition::BottomLeft
        } else {
            LicensePosition::BottomRight
        }
    }

    pub(crate) fn set_license_position(&self, license_position: LicensePosition) {
        let (valign, halign) = match license_position {
            LicensePosition::TopLeft => (gtk::Align::Start, gtk::Align::Start),
            LicensePosition::TopRight => (gtk::Align::Start, gtk::Align::End),
            LicensePosition::BottomRight => (gtk::Align::End, gtk::Align::End),
            LicensePosition::BottomLeft => (gtk::Align::End, gtk::Align::Start),
        };

        let license_label = self.imp().license_label.get();

        let _freeze_notify = license_label.freeze_notify();

        license_label.set_valign(valign);
        license_label.set_halign(halign);

        if self.license_position() != license_position {
            self.notify("license-position");
        }
    }

    pub(crate) fn viewport(&self) -> shumate::Viewport {
        self.imp().map.viewport().unwrap()
    }

    pub(crate) fn marker_location(&self) -> (f64, f64) {
        let imp = self.imp();
        (imp.marker.latitude(), imp.marker.longitude())
    }

    pub(crate) fn set_custom_marker(&self, marker: Option<gtk::Widget>) {
        let imp = self.imp();
        imp.marker.set_child(Some(&ui::MapMarker::from(
            &marker.unwrap_or_else(|| imp.marker_image.get().upcast()),
        )));
    }

    pub(crate) fn set_marker_position(&self, lat: f64, lon: f64) {
        self.imp().marker.set_location(lat, lon);
    }

    pub(crate) fn center_marker(&self, zoom_level: f64) {
        let viewport = self.viewport();
        viewport.set_zoom_level(zoom_level);

        let marker = &self.imp().marker;
        viewport.set_location(marker.latitude(), marker.longitude());
    }
}
