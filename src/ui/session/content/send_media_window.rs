use std::cell::OnceCell;
use std::cell::RefCell;

use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/send_media_window.ui")]
    pub(crate) struct SendMediaWindow {
        pub(super) chat: glib::WeakRef<model::Chat>,
        pub(super) path: OnceCell<String>,
        pub(super) emoji_chooser: RefCell<Option<gtk::EmojiChooser>>,
        #[template_child]
        pub(super) picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub(super) caption_entry: TemplateChild<ui::MessageEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SendMediaWindow {
        const NAME: &'static str = "PaplSendMediaWindow";
        type Type = super::SendMediaWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async(
                "send-media-window.send-message",
                None,
                |widget, _, _| async move {
                    widget.send_message(false).await;
                },
            );
            klass.install_action_async(
                "send-media-window.send-as-file",
                None,
                |widget, _, _| async move {
                    widget.send_message(true).await;
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SendMediaWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            self.caption_entry
                .connect_activate(clone!(@weak obj => move |_| {
                    obj.activate_action("send-media-window.send-message", None).unwrap()
                }));

            self.caption_entry
                .connect_emoji_button_press(clone!(@weak obj => move |_, button| {
                    obj.show_emoji_chooser(&button);
                }));
        }

        fn dispose(&self) {
            if let Some(emoji_chooser) = self.emoji_chooser.take() {
                emoji_chooser.unparent();
            }
        }
    }

    impl WidgetImpl for SendMediaWindow {}
    impl WindowImpl for SendMediaWindow {}
    impl AdwWindowImpl for SendMediaWindow {}
}

glib::wrapper! {
    pub(crate) struct SendMediaWindow(ObjectSubclass<imp::SendMediaWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl SendMediaWindow {
    pub(crate) fn new(parent: &gtk::Window, chat: &model::Chat, path: String) -> Self {
        let obj: Self = glib::Object::builder()
            .property("transient-for", parent)
            .build();
        let imp = obj.imp();

        imp.picture.set_filename(Some(&path));
        imp.caption_entry.set_chat(Some(chat.clone()));

        imp.chat.set(Some(chat));
        imp.path.set(path).unwrap();

        obj
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

    async fn send_message(&self, send_as_file: bool) {
        use tdlib::enums::*;
        use tdlib::types::*;

        let imp = self.imp();

        let chat = imp.chat.upgrade().unwrap();
        let chat_id = chat.id();
        let client_id = chat.session_().client_().id();
        let path = imp.path.get().unwrap().clone();

        let paintable = imp.picture.paintable().unwrap();
        let width = paintable.intrinsic_width();
        let height = paintable.intrinsic_height();
        let caption = imp.caption_entry.as_markdown().await;
        let self_destruct_type = Some(MessageSelfDestructType::Timer(
            MessageSelfDestructTypeTimer::default(),
        ));

        let file = InputFile::Local(InputFileLocal { path });
        let content = if send_as_file {
            InputMessageContent::InputMessageDocument(InputMessageDocument {
                document: file,
                thumbnail: None,
                disable_content_type_detection: true,
                caption,
            })
        } else {
            InputMessageContent::InputMessagePhoto(InputMessagePhoto {
                photo: file,
                thumbnail: None,
                added_sticker_file_ids: vec![],
                width,
                height,
                caption,
                self_destruct_type,
                has_spoiler: false,
            })
        };

        let reply_to = Some(MessageReplyTo::Message(MessageReplyToMessage {
            chat_id,
            message_id: 0,
        }));

        // TODO: maybe show an error dialog when this fails?
        if tdlib::functions::send_message(chat_id, 0, reply_to, None, content, client_id)
            .await
            .is_ok()
        {
            self.close();
        }
    }
}
