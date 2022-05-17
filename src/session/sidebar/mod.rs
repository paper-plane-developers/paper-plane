mod avatar;
mod row;
mod selection;
mod session_switcher;

use self::row::Row;
use self::selection::Selection;
use self::session_switcher::SessionSwitcher;

use gettextrs::gettext;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use tdlib::{enums, functions};

use crate::session::{Chat, ChatType, User};
use crate::utils::spawn;
use crate::Session;

pub(crate) use self::avatar::Avatar;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};

    use crate::session::components::Avatar as ComponentsAvatar;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar.ui")]
    pub(crate) struct Sidebar {
        pub(super) compact: Cell<bool>,
        pub(super) selected_chat: RefCell<Option<Chat>>,
        pub(super) session: RefCell<Option<Session>>,
        pub(super) filter: RefCell<Option<gtk::CustomFilter>>,
        pub(super) searched_chats: RefCell<Vec<i64>>,
        pub(super) searched_users: RefCell<Vec<i64>>,
        pub(super) already_searched_users: RefCell<Vec<i64>>,
        pub(super) row_menu: OnceCell<gtk::PopoverMenu>,
        #[template_child]
        pub(super) header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(super) session_switcher: TemplateChild<SessionSwitcher>,
        #[template_child]
        pub(super) search_bar: TemplateChild<gtk::SearchBar>,
        #[template_child]
        pub(super) search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub(super) selection: TemplateChild<Selection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            ComponentsAvatar::static_type();
            Row::static_type();
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl Sidebar {
        #[template_callback]
        async fn list_activate(&self, pos: u32) {
            let instance = self.instance();
            self.selection.set_selected_position(pos);

            instance.emit_by_name::<()>("list-activated", &[]);

            if let Some(item) = self.selection.selected_item() {
                if let Some(chat) = item.downcast_ref::<Chat>() {
                    instance.set_selected_chat(Some(chat.clone()));
                } else if let Some(user) = item.downcast_ref::<User>() {
                    // Create a chat with the user and then select the created chat
                    let user_id = user.id();
                    let session = user.session();
                    let client_id = session.client_id();
                    let result = functions::create_private_chat(user_id, false, client_id).await;

                    if let Ok(enums::Chat::Chat(chat)) = result {
                        let chat = session.chat_list().get(chat.id);
                        instance.set_selected_chat(Some(chat));
                    }
                }

                return;
            }

            instance.set_selected_chat(None);
        }
    }

    impl ObjectImpl for Sidebar {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("list-activated", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        "compact",
                        "Compact",
                        "Wheter a compact view is used or not",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "selected-chat",
                        "Selected Chat",
                        "The selected chat in this sidebar",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
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
            self.parent_constructed(obj);

            self.search_entry
                .connect_search_changed(clone!(@weak obj => move |entry| {
                    let query = entry.text().to_string();
                    spawn(clone!(@weak obj => async move {
                        obj.search(query).await;
                    }));
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
    pub(crate) struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget;
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Sidebar")
    }

    pub(crate) fn row_menu(&self) -> &gtk::PopoverMenu {
        self.imp().row_menu.get_or_init(|| {
            gtk::Builder::from_resource("/com/github/melix99/telegrand/ui/sidebar-row-menu.ui")
                .object::<gtk::PopoverMenu>("menu")
                .unwrap()
        })
    }

    pub(crate) fn begin_chats_search(&self) {
        let imp = self.imp();
        imp.search_bar.set_search_mode(true);
        imp.search_entry.grab_focus();
    }

    async fn search(&self, query: String) {
        let imp = self.imp();
        imp.searched_chats.borrow_mut().clear();
        imp.searched_users.borrow_mut().clear();
        imp.already_searched_users.borrow_mut().clear();

        if query.is_empty() {
            if let Some(filter) = imp.filter.borrow().as_ref() {
                filter.changed(gtk::FilterChange::Different);
            }
        } else {
            let client_id = self
                .session()
                .expect("The session needs to be set to be able to search")
                .client_id();

            // Search chats
            let result = functions::search_chats(query.clone(), 100, client_id).await;
            if let Ok(enums::Chats::Chats(chats)) = result {
                if let Some(filter) = imp.filter.borrow().as_ref() {
                    let session = self
                        .session()
                        .expect("The session needs to be set to be able to search");
                    let chat_list = session.chat_list();

                    // This will hold the own user id if the user has searched for the own
                    // chat.
                    let maybe_own_user_id = if gettext("Saved Messages")
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                    {
                        Some(session.me().id())
                    } else {
                        None
                    };

                    imp.already_searched_users.borrow_mut().extend(
                        chats
                            .chat_ids
                            .iter()
                            .map(|id| chat_list.get(*id))
                            .filter_map(|chat| match chat.type_() {
                                ChatType::Private(user) => Some(user.id()),
                                _ => None,
                            })
                            .chain(maybe_own_user_id.into_iter()),
                    );

                    imp.searched_chats.borrow_mut().extend(
                        chats
                            .chat_ids
                            .into_iter()
                            // The own user id is the same as the own chat id, so we can
                            // chain this here.
                            .chain(maybe_own_user_id.map(|id| id as i64).into_iter()),
                    );
                    filter.changed(gtk::FilterChange::Different);
                }
            }

            // Search contacts
            let result = functions::search_contacts(query, 100, client_id).await;
            if let Ok(enums::Users::Users(users)) = result {
                if let Some(filter) = imp.filter.borrow().as_ref() {
                    imp.searched_users.borrow_mut().extend(users.user_ids);
                    filter.changed(gtk::FilterChange::Different);
                }
            }
        }
    }

    fn selected_chat(&self) -> Option<Chat> {
        self.imp().selected_chat.borrow().clone()
    }

    fn set_selected_chat(&self, selected_chat: Option<Chat>) {
        if self.selected_chat() == selected_chat {
            return;
        }

        let imp = self.imp();
        imp.selection
            .set_selected_item(selected_chat.clone().map(Chat::upcast));

        imp.selected_chat.replace(selected_chat);
        self.notify("selected-chat");
    }

    pub(crate) fn set_session(&self, session: Option<Session>) {
        if self.session() == session {
            return;
        }

        let imp = self.imp();

        if let Some(ref session) = session {
            // Merge ChatList and UserList into a single list model
            let list = gio::ListStore::new(gio::ListModel::static_type());
            list.append(session.chat_list());
            list.append(session.user_list());
            let model = gtk::FlattenListModel::new(Some(&list));

            let filter = gtk::CustomFilter::new(
                clone!(@weak self as obj => @default-return false, move |item| {
                    let imp = obj.imp();
                    let is_searching = !imp.search_entry.text().is_empty();

                    if is_searching {
                        if let Some(chat) = item.downcast_ref::<Chat>() {
                            imp.searched_chats.borrow().contains(&chat.id())
                        } else if let Some(user) = item.downcast_ref::<User>() {
                            // Show searched users, but only the ones that haven't
                            // already been searched by the chats search
                            !imp.already_searched_users.borrow().contains(&user.id())
                                && imp.searched_users.borrow().contains(&user.id())
                        } else {
                            false
                        }
                    } else if let Some(chat) = item.downcast_ref::<Chat>() {
                        chat.order() > 0
                    } else {
                        false
                    }
                }),
            );
            let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
                let chat1 = obj1.downcast_ref::<Chat>();
                let chat2 = obj2.downcast_ref::<Chat>();

                // Always show chats first and then users
                if let Some(chat1) = chat1 {
                    if let Some(chat2) = chat2 {
                        chat2.order().cmp(&chat1.order()).into()
                    } else {
                        gtk::Ordering::Smaller
                    }
                } else if chat2.is_some() {
                    gtk::Ordering::Larger
                } else {
                    gtk::Ordering::Equal
                }
            });

            session.chat_list().connect_positions_changed(
                clone!(@weak filter, @weak sorter => move |_| {
                    filter.changed(gtk::FilterChange::Different);
                    sorter.changed(gtk::SorterChange::Different);
                }),
            );

            let filter_model = gtk::FilterListModel::new(Some(&model), Some(&filter));
            let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));

            imp.selection.set_model(Some(sort_model.upcast()));
            imp.filter.replace(Some(filter));
        }

        imp.session.replace(session);
        self.notify("session");
    }

    pub(crate) fn session(&self) -> Option<Session> {
        self.imp().session.borrow().to_owned()
    }

    pub(crate) fn set_sessions(&self, sessions: &gtk::SelectionModel, this_session: &Session) {
        self.imp()
            .session_switcher
            .set_sessions(sessions, this_session);
    }

    pub(crate) fn connect_list_activated<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("list-activated", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);

            None
        })
    }
}
