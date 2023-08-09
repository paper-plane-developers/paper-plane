use std::cell::RefCell;
mod file_status;
mod status_indicator;

use file_status::FileStatus;
use file_status::FileStatus::*;
use glib::clone;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use status_indicator::StatusIndicator;
use tdlib::enums::MessageContent;
use tdlib::types::File;

use super::base::MessageBaseExt;
use crate::session::content::message_row::MessageBase;
use crate::session::content::message_row::MessageBaseImpl;
use crate::session::content::message_row::MessageBubble;
use crate::tdlib::Message;
use crate::utils::parse_formatted_text;
use crate::utils::spawn;
use crate::Session;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/message_row/document/mod.ui")]
    pub(crate) struct MessageDocument {
        pub(super) bindings: RefCell<Vec<gtk::ExpressionWatch>>,
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) status_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) message: RefCell<Option<Message>>,
        #[template_child]
        pub(super) message_bubble: TemplateChild<MessageBubble>,
        #[template_child]
        pub(super) file_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) click: TemplateChild<gtk::GestureClick>,
        #[template_child]
        pub(super) file_thumbnail_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub(super) status_indicator: TemplateChild<StatusIndicator>,
        #[template_child]
        pub(super) file_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) file_size_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageDocument {
        const NAME: &'static str = "MessageDocument";
        type Type = super::MessageDocument;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageDocument {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<glib::Object>("message")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "message" => obj.set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => self.message.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageDocument {}
    impl MessageBaseImpl for MessageDocument {}
}

glib::wrapper! {
    pub(crate) struct MessageDocument(ObjectSubclass<imp::MessageDocument>)
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessageDocument {
    type Message = Message;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        let mut bindings = imp.bindings.borrow_mut();

        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        imp.message_bubble.update_from_message(&message, false);

        let handler_id =
            message.connect_content_notify(clone!(@weak self as obj => move |message, _| {
                obj.update_document(message);
            }));
        imp.handler_id.replace(Some(handler_id));
        self.update_document(&message);

        imp.message.replace(Some(message));
        self.notify("message");
    }
}

impl MessageDocument {
    fn update_document(&self, message: &Message) {
        if let MessageContent::MessageDocument(data) = message.content().0 {
            let imp = self.imp();

            let message_text = parse_formatted_text(data.caption);
            imp.message_bubble.set_label(message_text);

            imp.file_name_label.set_label(&data.document.file_name);

            let session = message.chat().session();

            self.try_load_thumbnail(message);
            self.update_status(data.document.document, session);
        }
    }

    fn update_status(&self, file: File, session: Session) -> FileStatus {
        let status = FileStatus::from(&file);

        let size = file.size.max(file.expected_size) as u64;

        self.update_size_label(status, size);
        self.update_button(file, session, status);

        status
    }

    fn update_button(&self, file: File, session: Session, status: FileStatus) {
        let imp = self.imp();
        let click = &*imp.click;
        let indicator = &*imp.status_indicator;
        let file_id = file.id;

        let handler_id = match status {
            Downloading(_progress) | Uploading(_progress) => {
                return;
                // Show loading indicator
            }
            CanBeDownloaded => {
                // Download file
                indicator.set_status(CanBeDownloaded);
                click.connect_released(clone!(@weak self as obj, @weak session => move |click, _, _, _| {
                    // TODO: Fix bug mentioned here
                    // https://github.com/paper-plane-developers/paper-plane/pull/372#discussion_r968841370
                    session.download_file_with_updates(file_id, clone!(@weak obj, @weak session => move |file| {
                        obj.update_status(file, session);
                    }));

                    obj.imp().status_indicator.set_status(Downloading(0.0));
                    let handler_id = click.connect_released(clone!(@weak session => move |_, _, _, _| {
                        session.cancel_download_file(file_id);
                    }));
                    if let Some(handler_id) = obj.imp().status_handler_id.replace(Some(handler_id)) {
                        click.disconnect(handler_id);
                    }
                }))
            }
            Downloaded => {
                // Open file
                indicator.set_status(Downloaded);
                if imp.file_thumbnail_picture.file().is_some() {
                    indicator.set_visible(false);
                }
                let gio_file = gio::File::for_path(&file.local.path);
                click.connect_released(move |_, _, _, _| {
                    if let Err(err) = gio::AppInfo::launch_default_for_uri(
                        &gio_file.uri(),
                        gio::AppLaunchContext::NONE,
                    ) {
                        log::error!("Error: {}", err);
                    }
                })
            }
        };

        if let Some(handler_id) = imp.status_handler_id.replace(Some(handler_id)) {
            click.disconnect(handler_id);
        }
    }

    fn update_size_label(&self, status: FileStatus, size: u64) {
        let size_label = &self.imp().file_size_label;

        match status {
            Downloading(progress) | Uploading(progress) => {
                let downloaded = glib::format_size((size as f64 * progress) as u64);
                let full_size = glib::format_size(size);

                size_label.set_label(&format!("{downloaded} / {full_size}"));
            }
            CanBeDownloaded | Downloaded => {
                size_label.set_label(&glib::format_size(size));
            }
        }
    }

    fn try_load_thumbnail(&self, message: &Message) {
        if let MessageContent::MessageDocument(data) = message.content().0 {
            let imp = self.imp();
            if let Some(thumbnail) = data.document.thumbnail {
                imp.status_indicator.set_masked(false);
                imp.file_thumbnail_picture.set_visible(true);
                imp.file_box.add_css_class("with-thumbnail");
                if thumbnail.file.local.is_downloading_completed {
                    imp.file_thumbnail_picture
                        .set_filename(Some(&thumbnail.file.local.path));
                } else {
                    if let Some(minithumbnail) = data.document.minithumbnail {
                        let minithumbnail = gdk::Texture::from_bytes(&glib::Bytes::from_owned(
                            glib::base64_decode(&minithumbnail.data),
                        ))
                        .unwrap();
                        imp.file_thumbnail_picture
                            .set_paintable(Some(&minithumbnail));
                    }

                    let session = message.chat().session();
                    spawn(clone!(@weak self as obj => async move {
                        if let Ok(file) = session.download_file(thumbnail.file.id).await
                        {
                            obj.imp()
                                .file_thumbnail_picture
                                .set_filename(Some(&file.local.path));
                        }
                    }));
                }
            } else {
                imp.status_indicator.set_masked(true);
                imp.file_thumbnail_picture.set_visible(false);
                imp.file_thumbnail_picture
                    .set_paintable(gdk::Paintable::NONE);
            }
        }
    }
}
