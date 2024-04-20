use std::sync::OnceLock;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct MapMarker;

    #[glib::object_subclass]
    impl ObjectSubclass for MapMarker {
        const NAME: &'static str = "PaplMapMarker";
        type Type = super::MapMarker;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for MapMarker {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<gtk::Widget>("marker-widget")
                        .explicit_notify()
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "marker-widget" => self.obj().set_marker_widget(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "marker-widget" => self.obj().marker_widget().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            if let Some(marker_widget) = self.obj().marker_widget() {
                marker_widget.unparent();
            }
        }
    }

    impl WidgetImpl for MapMarker {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match self.obj().marker_widget() {
                Some(marker_widget) => {
                    let (min, nat, min_base, nat_base) =
                        marker_widget.measure(orientation, for_size);
                    match orientation {
                        gtk::Orientation::Vertical => (min * 2, nat * 2, min_base, nat_base),
                        _ => (min, nat, min_base, nat_base),
                    }
                }
                None => (0, 0, -1, -1),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(marker_widget) = self.obj().marker_widget() {
                marker_widget.allocate(width, height, baseline, None);
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MapMarker(ObjectSubclass<imp::MapMarker>)
        @extends gtk::Widget;
}

impl<W: IsA<gtk::Widget>> From<&W> for MapMarker {
    fn from(widget: &W) -> Self {
        glib::Object::builder()
            .property("marker-widget", widget)
            .build()
    }
}

impl MapMarker {
    pub(crate) fn marker_widget(&self) -> Option<gtk::Widget> {
        self.first_child()
    }

    pub(crate) fn set_marker_widget(&self, marker_widget: Option<gtk::Widget>) {
        if self.marker_widget() == marker_widget {
            return;
        }

        if let Some(old_marker_widget) = self.first_child() {
            old_marker_widget.unparent();
        }

        if let Some(marker_widget) = marker_widget {
            marker_widget.set_valign(gtk::Align::Start);
            marker_widget.insert_after(self, gtk::Widget::NONE);
        }
        self.notify("marker-widget");
    }
}
