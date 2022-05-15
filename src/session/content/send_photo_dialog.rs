use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::{InputFile, InputMessageContent};
use tdlib::functions;
use tdlib::types::{InputFileLocal, InputMessagePhoto};

use crate::expressions;
use crate::session::components::MessageEntry;
use crate::tdlib::Chat;
use crate::utils::spawn;

mod imp {
    use super::*;
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-send-photo-dialog.ui")]
    pub(crate) struct SendPhotoDialog {
        pub(super) chat: OnceCell<Chat>,
        pub(super) path: OnceCell<String>,
        pub(super) emoji_chooser: RefCell<Option<gtk::EmojiChooser>>,
        #[template_child]
        pub(super) header_bar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub(super) picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub(super) caption_entry: TemplateChild<MessageEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SendPhotoDialog {
        const NAME: &'static str = "ContentSendPhotoDialog";
        type Type = super::SendPhotoDialog;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "send-photo-dialog.send-message",
                None,
                move |widget, _, _| {
                    spawn(clone!(@weak widget => async move {
                        widget.send_message().await;
                    }));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SendPhotoDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.caption_entry
                .connect_activate(clone!(@weak obj => move |_| {
                    obj.activate_action("send-photo-dialog.send-message", None).unwrap()
                }));

            self.caption_entry
                .connect_emoji_button_press(clone!(@weak obj => move |_, button| {
                    obj.show_emoji_chooser(&button);
                }));
        }

        fn dispose(&self, _obj: &Self::Type) {
            if let Some(emoji_chooser) = self.emoji_chooser.take() {
                emoji_chooser.unparent();
            }
        }
    }

    impl WidgetImpl for SendPhotoDialog {}
    impl WindowImpl for SendPhotoDialog {}
}

glib::wrapper! {
    pub(crate) struct SendPhotoDialog(ObjectSubclass<imp::SendPhotoDialog>)
        @extends gtk::Widget, gtk::Window;
}

impl SendPhotoDialog {
    pub(crate) fn new(parent_window: &Option<gtk::Window>, chat: Chat, path: String) -> Self {
        let send_photo_dialog: Self =
            glib::Object::new(&[("transient-for", parent_window)]).unwrap();
        let imp = send_photo_dialog.imp();

        let chat_expression = gtk::ConstantExpression::new(&chat);
        expressions::chat_display_name(&chat_expression).bind(
            &send_photo_dialog,
            "title",
            glib::Object::NONE,
        );

        imp.picture.set_filename(Some(&path));
        imp.chat.set(chat).unwrap();
        imp.path.set(path).unwrap();

        send_photo_dialog
    }

    fn show_emoji_chooser(&self, parent: &impl IsA<gtk::Widget>) {
        let imp = self.imp();
        let mut emoji_chooser = imp.emoji_chooser.borrow_mut();
        if emoji_chooser.is_none() {
            let chooser = gtk::EmojiChooser::new();
            chooser.set_parent(parent);
            chooser.connect_emoji_picked(clone!(@weak self as obj => move |_, emoji| {
                obj.imp().caption_entry.insert_at_cursor(emoji);
            }));
            chooser.connect_hide(clone!(@weak self as obj => move |_| {
                obj.imp().caption_entry.grab_focus();
            }));
            *emoji_chooser = Some(chooser);
        }
        emoji_chooser.as_ref().unwrap().popup();
    }

    async fn send_message(&self) {
        let imp = self.imp();

        let chat = imp.chat.get().unwrap();
        let chat_id = chat.id();
        let client_id = chat.session().client_id();
        let path = imp.path.get().unwrap().clone();

        let paintable = imp.picture.paintable().unwrap();
        let width = paintable.intrinsic_width();
        let height = paintable.intrinsic_height();

        let file = InputFile::Local(InputFileLocal { path });
        let content = InputMessageContent::InputMessagePhoto(InputMessagePhoto {
            photo: file,
            thumbnail: None,
            added_sticker_file_ids: vec![],
            width,
            height,
            caption: imp.caption_entry.formatted_text().map(|f| f.0),
            ttl: 0,
        });

        // TODO: maybe show an error dialog when this fails?
        if functions::send_message(chat_id, 0, 0, None, content, client_id)
            .await
            .is_ok()
        {
            self.close();
        }
    }
}
