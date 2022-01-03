use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::session::{
    chat::SponsoredMessageList,
    content::{ChatActionBar, ItemRow, UserDialog},
    Chat, ChatType
};

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
        pub sponsored_message_list: RefCell<Option<SponsoredMessageList>>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
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
                let dialog = UserDialog::new(&self.parent_window(), &user);
                dialog.show();
            }
        }
    }

    fn parent_window(&self) -> Option<gtk::Window> {
        self.root()?.downcast().ok()
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

            let list = gio::ListStore::new(gio::ListModel::static_type());
            list.append(&chat.history());

            if matches!(chat.type_(), ChatType::Supergroup(supergroup) if supergroup.is_channel()) {
                let mut sponsored_message_list_ref = self_.sponsored_message_list.borrow_mut();
                let sponsored_message_list = match *sponsored_message_list_ref {
                    None => {
                        let sponsored_message_list = SponsoredMessageList::new();
                        *sponsored_message_list_ref = Some(sponsored_message_list);
                        sponsored_message_list_ref.as_ref().unwrap()
                    }
                    Some(ref sponsored_message_list) => {
                        sponsored_message_list.clear();
                        sponsored_message_list
                    }
                };
                sponsored_message_list.fetch(chat);
                list.append(sponsored_message_list);
            }

            let model = gtk::FlattenListModel::new(Some(&list));
            let selection = gtk::NoSelection::new(Some(&model));
            self_.list_view.set_model(Some(&selection));
        }

        self_.chat.replace(chat);
        self.notify("chat");

        let adj = self_.list_view.vadjustment().unwrap();
        self.load_older_messages(&adj);
        if let Some(chat) = self.chat() {
            let title = Chat::title_expression(&chat);
            let subtitle = Chat::subtitle_expression(&chat);
            title.bind(&*self_.window_title, "title", gtk::NONE_WIDGET);
            subtitle.bind(&*self_.window_title, "subtitle", gtk::NONE_WIDGET);
        }
    }
}
