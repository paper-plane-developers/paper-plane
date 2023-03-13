use adw::prelude::*;
use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::ChatMemberStatus;
use tdlib::functions;

use super::{ChatActionBar, ChatInfoWindow};
use crate::components::MessageListView;
use crate::expressions;
use crate::tdlib::{Chat, ChatType};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-history.ui")]
    pub(crate) struct ChatHistory {
        pub(super) compact: Cell<bool>,
        pub(super) chat: RefCell<Option<Chat>>,
        #[template_child]
        pub(super) window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(super) message_list_view: TemplateChild<MessageListView>,
        #[template_child]
        pub(super) chat_action_bar: TemplateChild<ChatActionBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatHistory {
        const NAME: &'static str = "ContentChatHistory";
        type Type = super::ChatHistory;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("chat-history.view-info", None, move |widget, _, _| {
                widget.open_info_dialog();
            });
            klass.install_action(
                "chat-history.reply",
                Some("x"),
                move |widget, _, variant| {
                    let message_id = variant.and_then(|v| v.get()).unwrap();
                    widget.imp().chat_action_bar.reply_to_message_id(message_id);
                },
            );
            klass.install_action("chat-history.edit", Some("x"), move |widget, _, variant| {
                let message_id = variant.and_then(|v| v.get()).unwrap();
                widget.imp().chat_action_bar.edit_message_id(message_id);
            });
            klass.install_action_async(
                "chat-history.leave-chat",
                None,
                |widget, _, _| async move {
                    widget.show_leave_chat_dialog().await;
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatHistory {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("compact").build(),
                    glib::ParamSpecObject::builder::<Chat>("chat")
                        .explicit_notify()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "compact" => {
                    let compact = value.get().unwrap();
                    self.compact.set(compact);
                }
                "chat" => {
                    let chat = value.get().unwrap();
                    obj.set_chat(chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_expressions();
        }
    }

    impl WidgetImpl for ChatHistory {}
    impl BinImpl for ChatHistory {}
}

glib::wrapper! {
    pub(crate) struct ChatHistory(ObjectSubclass<imp::ChatHistory>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatHistory {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    fn setup_expressions(&self) {
        let chat_expression = Self::this_expression("chat");

        // Chat title
        expressions::chat_display_name(&chat_expression).bind(
            &*self.imp().window_title,
            "title",
            Some(self),
        );
    }

    fn open_info_dialog(&self) {
        if let Some(chat) = self.chat() {
            ChatInfoWindow::new(&self.parent_window(), &chat).present();
        }
    }

    async fn show_leave_chat_dialog(&self) {
        if let Some(chat) = self.chat() {
            let dialog = adw::MessageDialog::new(
                Some(&self.parent_window().unwrap()),
                Some(&gettext("Leave chat?")),
                Some(&gettext("Do you want to leave this chat?")),
            );
            dialog.add_responses(&[("no", &gettext("_No")), ("yes", &gettext("_Yes"))]);
            dialog.set_default_response(Some("no"));
            dialog.set_close_response("no");
            dialog.set_response_appearance("yes", adw::ResponseAppearance::Destructive);
            let response = dialog.choose_future().await;
            if response == "yes" {
                let result = functions::leave_chat(chat.id(), chat.session().client_id()).await;
                if let Err(e) = result {
                    log::warn!("Failed to leave chat: {:?}", e);
                } else {
                    // Unselect recently left chat
                    chat.session().imp().sidebar.get().set_selected_chat(None);
                }
            }
        }
    }

    fn parent_window(&self) -> Option<gtk::Window> {
        self.root()?.downcast().ok()
    }

    pub(crate) fn handle_paste_action(&self) {
        self.imp().chat_action_bar.handle_paste_action();
    }

    pub(crate) fn chat(&self) -> Option<Chat> {
        self.imp().chat.borrow().clone()
    }

    pub(crate) fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let imp = self.imp();

        if let Some(ref chat) = chat {
            self.action_set_enabled(
                "chat-history.leave-chat",
                match chat.type_() {
                    ChatType::BasicGroup(data) => data.status().0 != ChatMemberStatus::Left,
                    ChatType::Supergroup(data) => data.status().0 != ChatMemberStatus::Left,
                    _ => false,
                },
            );

            imp.message_list_view.load_messages(chat);
        }

        imp.chat.replace(chat);
        self.notify("chat");
    }
}
