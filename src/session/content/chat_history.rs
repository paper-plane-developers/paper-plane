use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::session::{
    chat::SponsoredMessage,
    content::{ChatActionBar, ItemRow, UserDialog},
    Chat, ChatType, Session,
};
use crate::spawn;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-history.ui")]
    pub struct ChatHistory {
        pub compact: Cell<bool>,
        pub chat: RefCell<Option<Chat>>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatHistory {
        const NAME: &'static str = "ContentChatHistory";
        type Type = super::ChatHistory;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ItemRow::static_type();
            ChatActionBar::static_type();
            Self::bind_template(klass);

            klass.install_action("chat-history.view-info", None, move |widget, _, _| {
                widget.open_info_dialog();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatHistory {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boolean(
                        "compact",
                        "Compact",
                        "Wheter a compact view is used or not",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "chat",
                        "Chat",
                        "The chat currently shown",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
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

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let adj = self.list_view.vadjustment().unwrap();
            adj.connect_value_changed(clone!(@weak obj => move |adj| {
                obj.load_older_messages(adj);
            }));
        }
    }

    impl WidgetImpl for ChatHistory {}
    impl BinImpl for ChatHistory {}
}

glib::wrapper! {
    pub struct ChatHistory(ObjectSubclass<imp::ChatHistory>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatHistory {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChatHistory")
    }

    fn load_older_messages(&self, adj: &gtk::Adjustment) {
        if adj.value() < adj.page_size() * 2.0 || adj.upper() <= adj.page_size() * 2.0 {
            if let Some(chat) = self.chat() {
                chat.history().load_older_messages();
            }
        }
    }

    fn open_info_dialog(&self) {
        if let Some(chat) = self.chat() {
            if let ChatType::Private(user) = chat.type_() {
                let dialog = UserDialog::new(&self.parent_window(), user);
                dialog.show();
            }
        }
    }

    fn parent_window(&self) -> Option<gtk::Window> {
        self.root()?.downcast().ok()
    }

    fn request_sponsored_message(&self, session: &Session, chat_id: i64, list: &gio::ListStore) {
        spawn!(clone!(@weak session, @weak list => async move {
            match SponsoredMessage::request(chat_id, &session).await {
                Ok(sponsored_message) => list.append(&sponsored_message),
                Err(e) => log::warn!("Failed to request a SponsoredMessage: {:?}", e),
            }
        }));
    }

    pub fn chat(&self) -> Option<Chat> {
        let self_ = imp::ChatHistory::from_instance(self);
        self_.chat.borrow().clone()
    }

    pub fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let self_ = imp::ChatHistory::from_instance(self);
        if let Some(ref chat) = chat {
            match chat.type_() {
                ChatType::Private(_) => self.action_set_enabled("chat-history.view-info", true),
                _ => self.action_set_enabled("chat-history.view-info", false),
            }

            // Request sponsored message, if needed
            let chat_history: gio::ListModel = if matches!(chat.type_(), ChatType::Supergroup(supergroup) if supergroup.is_channel())
            {
                let list = gio::ListStore::new(gio::ListModel::static_type());
                list.append(&chat.history());

                // We need to create a list here so that we can append the sponsored message
                // to the chat history in the GtkListView using a GtkFlattenListModel
                let sponsored_message_list = gio::ListStore::new(SponsoredMessage::static_type());
                list.append(&sponsored_message_list);
                self.request_sponsored_message(&chat.session(), chat.id(), &sponsored_message_list);

                gtk::FlattenListModel::new(Some(&list)).upcast()
            } else {
                chat.history().upcast()
            };

            let selection = gtk::NoSelection::new(Some(&chat_history));
            self_.list_view.set_model(Some(&selection));
        }

        self_.chat.replace(chat);
        self.notify("chat");

        let adj = self_.list_view.vadjustment().unwrap();
        self.load_older_messages(&adj);
    }
}
