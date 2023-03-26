use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, graphene, gsk};

use tdlib::types::ClosedVectorPath;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub(crate) struct VectorPath {
        pub(super) node: RefCell<Option<gsk::RenderNode>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VectorPath {
        const NAME: &'static str = "ComponentsVectorPath";
        type Type = super::VectorPath;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("vectorpath");
        }
    }

    impl ObjectImpl for VectorPath {}

    impl WidgetImpl for VectorPath {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();

            let factor = (widget.width() as f32).max(widget.height() as f32) / 512.0;
            snapshot.scale(factor, factor);

            if let Some(node) = &*self.node.borrow() {
                snapshot.append_node(node);
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct VectorPath(ObjectSubclass<imp::VectorPath>)
        @extends gtk::Widget;
}

impl VectorPath {
    pub fn new(outline: &[ClosedVectorPath]) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().node.replace(Self::path_node(outline));
        obj
    }

    fn path_node(outline: &[ClosedVectorPath]) -> Option<gsk::RenderNode> {
        use tdlib::enums::VectorPathCommand::{CubicBezierCurve, Line};
        use tdlib::types::VectorPathCommandCubicBezierCurve as Curve;

        let snapshot = gtk::Snapshot::new();
        let context = snapshot.append_cairo(&graphene::Rect::new(0.0, 0.0, 512.0, 512.0));

        context.set_source_rgba(0.5, 0.5, 0.5, 0.4);

        for closed_path in outline {
            context.new_sub_path();
            for command in &closed_path.commands {
                match command {
                    Line(line) => {
                        let e = &line.end_point;
                        context.line_to(e.x, e.y);
                    }
                    CubicBezierCurve(curve) => {
                        let Curve {
                            start_control_point: sc,
                            end_control_point: ec,
                            end_point: e,
                        } = curve;

                        context.curve_to(sc.x, sc.y, ec.x, ec.y, e.x, e.y);
                    }
                }
            }
            context.close_path();
        }
        _ = context.fill();

        snapshot.to_node()
    }
}
