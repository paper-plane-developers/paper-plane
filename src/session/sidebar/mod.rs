mod chat_row;

use self::chat_row::ChatRow;

use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::session::{Chat, ChatList};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar.ui")]
    pub struct Sidebar {
        pub compact: Cell<bool>,
        pub selected_chat: RefCell<Option<Chat>>,
        pub selection: RefCell<Option<gtk::SingleSelection>>,
        pub filter: OnceCell<gtk::CustomFilter>,
        pub sorter: OnceCell<gtk::CustomSorter>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub chat_list_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ChatRow::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Sidebar {
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
                        "chat-list",
                        "Chat List",
                        "A list of chats",
                        ChatList::static_type(),
                        glib::ParamFlags::WRITABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-chat",
                        "Selected Chat",
                        "The selected chat in this sidebar",
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
                "chat-list" => {
                    let chat_list = value.get().unwrap();
                    obj.set_chat_list(chat_list);
                }
                "selected-chat" => {
                    let selected_chat = value.get().unwrap();
                    obj.set_selected_chat(selected_chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "selected-chat" => obj.selected_chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Sidebar {}
    impl BinImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Sidebar")
    }

    pub fn freeze(&self) {
        let self_ = imp::Sidebar::from_instance(self);
        self_.menu_button.set_sensitive(false);
        self_.scrolled_window.set_sensitive(false);
    }

    pub fn set_chat_list(&self, chat_list: ChatList) {
        let self_ = imp::Sidebar::from_instance(self);
        let filter = gtk::CustomFilter::new(|item| {
            let order = item.downcast_ref::<Chat>().unwrap().order();
            order != 0
        });
        self_.filter.set(filter).unwrap();

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let order1 = obj1.downcast_ref::<Chat>().unwrap().order();
            let order2 = obj2.downcast_ref::<Chat>().unwrap().order();

            order2.cmp(&order1).into()
        });
        self_.sorter.set(sorter).unwrap();

        let filter = self_.filter.get().unwrap();
        let sorter = self_.sorter.get().unwrap();
        chat_list.connect_positions_changed(clone!(@weak filter, @weak sorter => move |_| {
            filter.changed(gtk::FilterChange::Different);
            sorter.changed(gtk::SorterChange::Different);
        }));

        let filter_model = gtk::FilterListModel::new(Some(&chat_list), Some(filter));
        let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(sorter));
        let selection = gtk::SingleSelection::new(Some(&sort_model));
        selection.set_autoselect(false);
        selection
            .bind_property("selected-item", self, "selected-chat")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self_.selection.replace(Some(selection));

        self_
            .chat_list_view
            .set_model(Some(self_.selection.borrow().as_ref().unwrap()));
    }

    fn selected_chat(&self) -> Option<Chat> {
        let self_ = imp::Sidebar::from_instance(self);
        self_.selected_chat.borrow().clone()
    }

    fn set_selected_chat(&self, selected_chat: Option<Chat>) {
        if self.selected_chat() == selected_chat {
            return;
        }

        // TODO: change the selection in the sidebar if it's
        // different from the current selection

        let self_ = imp::Sidebar::from_instance(self);
        if selected_chat.is_none() {
            self_
                .selection
                .borrow()
                .as_ref()
                .unwrap()
                .set_selected(gtk::INVALID_LIST_POSITION);
        }

        self_.selected_chat.replace(selected_chat);
        self.notify("selected-chat");
    }
}
