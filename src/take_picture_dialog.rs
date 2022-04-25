use std::os::unix::prelude::RawFd;

use ashpd::desktop::camera;
use ashpd::zbus;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use crate::{session::CameraPaintable, utils::spawn};

    use super::*;
    use adw::subclass::prelude::AdwWindowImpl;
    use gst::glib::clone;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/take-picture-dialog.ui")]
    pub(crate) struct TakePictureDialog {
        #[template_child]
        pub(super) picture: TemplateChild<gtk::Picture>,
        pub(super) paintable: CameraPaintable,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TakePictureDialog {
        const NAME: &'static str = "TakePictureDialog";
        type Type = super::TakePictureDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TakePictureDialog {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| vec![]);
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.picture.set_paintable(Some(&self.paintable));

            spawn(clone!(@weak obj => async move {
                obj.start_stream().await;
            }));
        }

        fn dispose(&self, obj: &Self::Type) {
            obj.stop_stream();
        }
    }

    impl WidgetImpl for TakePictureDialog {}
    impl WindowImpl for TakePictureDialog {}
    impl AdwWindowImpl for TakePictureDialog {}
}

glib::wrapper! {
    pub(crate) struct TakePictureDialog(ObjectSubclass<imp::TakePictureDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl TakePictureDialog {
    pub(crate) fn new(parent_window: &Option<gtk::Window>) -> Self {
        glib::Object::new(&[("transient-for", parent_window)])
            .expect("Failed to create TakePictureDialog")
    }

    async fn start_stream(&self) {
        let imp = self.imp();

        self.action_set_enabled("camera.stop", true);
        self.action_set_enabled("camera.start", false);
        match stream().await {
            Ok(stream_fd) => {
                println!("{stream_fd}");
                let node_id = camera::pipewire_node_id(stream_fd).await.unwrap();
                imp.paintable.set_pipewire_node_id(stream_fd, node_id);
            }
            Err(err) => {
                log::error!("Failed to start a camera stream {:#?}", err);
                self.stop_stream();
            }
        }
    }

    fn stop_stream(&self) {
        self.imp().paintable.close_pipeline();
    }
}

async fn stream() -> ashpd::Result<RawFd> {
    let connection = zbus::Connection::session().await?;
    let proxy = camera::CameraProxy::new(&connection).await?;
    proxy.access_camera().await?;
    proxy.open_pipe_wire_remote().await
}
