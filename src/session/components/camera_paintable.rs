// Taken from https://github.com/bilelmoussaoui/ashpd/blob/cb97f4442999803831fc71e355189448b41d1e8f/ashpd-demo/src/widgets/gst_paintable.rs

use gst::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    gdk,
    glib::{self, clone},
    graphene,
};
use std::os::unix::io::AsRawFd;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct CameraPaintable {
        pub pipeline: RefCell<Option<gst::Pipeline>>,
        pub sink_paintable: RefCell<Option<gdk::Paintable>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CameraPaintable {
        const NAME: &'static str = "CameraPaintable";
        type Type = super::CameraPaintable;
        type Interfaces = (gdk::Paintable,);
    }

    impl ObjectImpl for CameraPaintable {
        fn dispose(&self, paintable: &Self::Type) {
            paintable.close_pipeline();
        }
    }

    impl PaintableImpl for CameraPaintable {
        fn intrinsic_height(&self, _paintable: &Self::Type) -> i32 {
            if let Some(ref paintable) = *self.sink_paintable.borrow() {
                paintable.intrinsic_height()
            } else {
                0
            }
        }

        fn intrinsic_width(&self, _paintable: &Self::Type) -> i32 {
            if let Some(ref paintable) = *self.sink_paintable.borrow() {
                paintable.intrinsic_width()
            } else {
                0
            }
        }

        fn snapshot(
            &self,
            _paintable: &Self::Type,
            snapshot: &gdk::Snapshot,
            width: f64,
            height: f64,
        ) {
            if let Some(ref image) = *self.sink_paintable.borrow() {
                image.snapshot(snapshot, width, height);
            } else {
                let snapshot = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();
                snapshot.append_color(
                    &gdk::RGBA::BLACK,
                    &graphene::Rect::new(0f32, 0f32, width as f32, height as f32),
                );
            }
        }
    }
}

glib::wrapper! {
    pub struct CameraPaintable(ObjectSubclass<imp::CameraPaintable>) @implements gdk::Paintable;
}

impl CameraPaintable {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a CameraPaintable")
    }

    pub fn set_pipewire_fd<F: AsRawFd>(&self, fd: F) {
        let raw_fd = fd.as_raw_fd();
        log::debug!("Loading PipeWire FD: {}", raw_fd);
        let pipewire_element = gst::ElementFactory::make("pipewiresrc", None).unwrap();
        pipewire_element.set_property("fd", &raw_fd);
        self.init_pipeline(pipewire_element);
    }

    pub fn set_pipewire_node_id<F: AsRawFd>(&self, fd: F, node_id: Option<u32>) {
        let raw_fd = fd.as_raw_fd();
        let pipewire_element = gst::ElementFactory::make("pipewiresrc", None).unwrap();
        pipewire_element.set_property("fd", &raw_fd);
        if let Some(node) = node_id {
            log::debug!(
                "Loading PipeWire Node ID: {} with FD: {}",
                node.to_string(),
                raw_fd
            );
            pipewire_element.set_property("path", &node.to_string());
        } else {
            log::debug!("Loading PipeWire FD: {}", raw_fd);
        }
        self.init_pipeline(pipewire_element);
    }

    fn init_pipeline(&self, pipewire_src: gst::Element) {
        log::debug!("Init pipeline");
        let imp = self.imp();
        let pipeline = gst::Pipeline::new(None);

        let sink = gst::ElementFactory::make("gtk4paintablesink", None).unwrap();
        let paintable = sink.property::<gdk::Paintable>("paintable");

        paintable.connect_invalidate_contents(clone!(@weak self as pt => move |_| {
            pt.invalidate_contents();
        }));

        paintable.connect_invalidate_size(clone!(@weak self as pt => move |_| {
            pt.invalidate_size();
        }));
        imp.sink_paintable.replace(Some(paintable));

        let convert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let queue1 = gst::ElementFactory::make("queue", None).unwrap();
        let queue2 = gst::ElementFactory::make("queue", None).unwrap();
        pipeline
            .add_many(&[&pipewire_src, &queue1, &convert, &queue2, &sink])
            .unwrap();

        pipewire_src.link(&queue1).unwrap();
        queue1.link(&convert).unwrap();
        convert.link(&queue2).unwrap();
        queue2.link(&sink).unwrap();

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(move |_, msg| {
            if let gst::MessageView::Error(err) = msg.view() {
                log::error!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
            }
            glib::Continue(true)
        })
        .expect("Failed to add bus watch");
        pipeline.set_state(gst::State::Playing).unwrap();
        imp.pipeline.replace(Some(pipeline));
    }

    pub fn close_pipeline(&self) {
        log::debug!("Closing pipeline");
        if let Some(pipeline) = self.imp().pipeline.borrow_mut().take() {
            pipeline.set_state(gst::State::Null).unwrap();
        }
    }
}

impl Default for CameraPaintable {
    fn default() -> Self {
        Self::new()
    }
}
