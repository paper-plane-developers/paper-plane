mod avatar;
mod row;

use self::row::Row;

use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums, functions};

use crate::session::Chat;
use crate::utils::do_async;
use crate::Session;

pub use self::avatar::Avatar;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar.ui")]
    pub struct Sidebar {
        pub compact: Cell<bool>,
        pub selected_chat: RefCell<Option<Chat>>,
        pub session: RefCell<Option<Session>>,
        pub filter: RefCell<Option<gtk::CustomFilter>>,
        pub selection: RefCell<Option<gtk::SingleSelection>>,
        pub searched_chats: RefCell<Vec<i64>>,
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub search_bar: TemplateChild<gtk::SearchBar>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Row::static_type();
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
                        "selected-chat",
                        "Selected Chat",
                        "The selected chat in this sidebar",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
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
                "selected-chat" => {
                    let selected_chat = value.get().unwrap();
                    obj.set_selected_chat(selected_chat);
                }
                "session" => {
                    let session = value.get().unwrap();
                    obj.set_session(session);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "selected-chat" => obj.selected_chat().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.search_entry
                .connect_search_changed(clone!(@weak obj => move |entry| {
                    let query = entry.text().to_string();
                    obj.search(query);
                }));
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.header_bar.unparent();
            self.search_bar.unparent();
            self.scrolled_window.unparent();
        }
    }

    impl WidgetImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget;
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

    pub fn begin_chats_search(&self) {
        let self_ = imp::Sidebar::from_instance(self);
        self_.search_bar.set_search_mode(true);
        self_.search_entry.grab_focus();
    }

    fn search(&self, query: String) {
        let self_ = imp::Sidebar::from_instance(self);
        self_.searched_chats.borrow_mut().clear();

        if query.is_empty() {
            if let Some(filter) = self_.filter.borrow().as_ref() {
                filter.changed(gtk::FilterChange::Different);
            }
        } else {
            let client_id = self
                .session()
                .expect("The session needs to be set to be able to search")
                .client_id();
            do_async(
                glib::PRIORITY_DEFAULT_IDLE,
                async move {
                    functions::SearchChats::new()
                        .query(query)
                        .limit(100)
                        .send(client_id)
                        .await
                },
                clone!(@weak self as obj => move |result| async move {
                    if let Ok(enums::Chats::Chats(chats)) = result {
                        let self_ = imp::Sidebar::from_instance(&obj);

                        if let Some(filter) = self_.filter.borrow().as_ref() {
                            self_.searched_chats.borrow_mut().extend(chats.chat_ids);
                            filter.changed(gtk::FilterChange::Different);
                        }
                    }
                }),
            );
        }
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

    pub fn set_session(&self, session: Option<Session>) {
        if self.session() == session {
            return;
        }

        let self_ = imp::Sidebar::from_instance(self);

        if let Some(ref session) = session {
            let filter = gtk::CustomFilter::new(
                clone!(@weak self as obj => @default-return false, move |item| {
                    let self_ = imp::Sidebar::from_instance(&obj);
                    let is_searching = !self_.search_entry.text().is_empty();
                    let chat = item.downcast_ref::<Chat>().unwrap();

                    if is_searching {
                        self_.searched_chats.borrow().contains(&chat.id())
                    } else {
                        chat.order() > 0
                    }
                }),
            );
            let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
                let order1 = obj1.downcast_ref::<Chat>().unwrap().order();
                let order2 = obj2.downcast_ref::<Chat>().unwrap().order();

                order2.cmp(&order1).into()
            });

            session.chat_list().connect_positions_changed(
                clone!(@weak filter, @weak sorter => move |_| {
                    filter.changed(gtk::FilterChange::Different);
                    sorter.changed(gtk::SorterChange::Different);
                }),
            );

            let filter_model = gtk::FilterListModel::new(Some(session.chat_list()), Some(&filter));
            let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
            let selection = gtk::SingleSelection::new(Some(&sort_model));
            selection.set_autoselect(false);
            selection
                .bind_property("selected-item", self, "selected-chat")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            self_.list_view.set_model(Some(&selection));
            self_.filter.replace(Some(filter));
            self_.selection.replace(Some(selection));
        }

        self_.session.replace(session);
        self.notify("session");
    }

    pub fn session(&self) -> Option<Session> {
        let self_ = imp::Sidebar::from_instance(self);
        self_.session.borrow().to_owned()
    }
}
