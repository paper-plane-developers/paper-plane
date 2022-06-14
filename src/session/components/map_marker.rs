use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct MapMarker;

    #[glib::object_subclass]
    impl ObjectSubclass for MapMarker {
        const NAME: &'static str = "ComponentsMapMarker";
        type Type = super::MapMarker;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for MapMarker {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "marker-widget",
                    "Marker Widget",
                    "the widget to display as a map marker",
                    gtk::Widget::static_type(),
                    glib::ParamFlags::READWRITE
                        | glib::ParamFlags::CONSTRUCT_ONLY
                        | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "marker-widget" => obj.set_marker_widget(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "marker-widget" => obj.marker_widget().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, obj: &Self::Type) {
            if let Some(marker_widget) = obj.marker_widget() {
                marker_widget.unparent();
            }
        }
    }

    impl WidgetImpl for MapMarker {
        fn measure(
            &self,
            widget: &Self::Type,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            match widget.marker_widget() {
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

        fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
            if let Some(marker_widget) = widget.marker_widget() {
                marker_widget.allocate(width, height, baseline, None);
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MapMarker(ObjectSubclass<imp::MapMarker>)
        @extends gtk::Widget;
}

impl<W: glib::IsA<gtk::Widget>> From<&W> for MapMarker {
    fn from(widget: &W) -> Self {
        glib::Object::new(&[("marker-widget", widget)])
            .expect("Failed to create ComponentsMapMarker")
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
