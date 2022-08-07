mod avatar;
mod mini_thumbnail;
mod row;
mod selection;
mod session_switcher;

use self::row::Row;
use self::selection::Selection;
use self::session_switcher::SessionSwitcher;

use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::tdlib::Chat;
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
        pub(super) row_menu: OnceCell<gtk::PopoverMenu>,
        #[template_child]
        pub(super) header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(super) session_switcher: TemplateChild<SessionSwitcher>,
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
        fn list_activate(&self, pos: u32) {
            self.selection.set_selected_position(pos);

            let instance = self.instance();
            let chat = self
                .selection
                .selected_item()
                .map(|i| i.downcast().unwrap());
            instance.set_selected_chat(chat);

            instance.emit_by_name::<()>("list-activated", &[]);
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

        fn dispose(&self, _obj: &Self::Type) {
            self.header_bar.unparent();
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

    pub(crate) fn begin_chats_search(&self) {}

    pub(crate) fn selected_chat(&self) -> Option<Chat> {
        self.imp().selected_chat.borrow().clone()
    }

    pub(crate) fn set_selected_chat(&self, selected_chat: Option<Chat>) {
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
            let filter = gtk::CustomFilter::new(|item| {
                let chat = item.downcast_ref::<Chat>().unwrap();
                chat.order() > 0
            });
            let sorter = gtk::CustomSorter::new(|item1, item2| {
                let chat1 = item1.downcast_ref::<Chat>().unwrap();
                let chat2 = item2.downcast_ref::<Chat>().unwrap();
                chat2.order().cmp(&chat1.order()).into()
            });

            session.chat_list().connect_positions_changed(
                clone!(@weak filter, @weak sorter => move |_| {
                    filter.changed(gtk::FilterChange::Different);
                    sorter.changed(gtk::SorterChange::Different);
                }),
            );

            let filter_model = gtk::FilterListModel::new(Some(session.chat_list()), Some(&filter));
            let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));

            imp.selection.set_model(Some(sort_model.upcast()));
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
