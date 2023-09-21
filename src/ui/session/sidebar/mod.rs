pub(crate) mod avatar;
pub(crate) mod mini_thumbnail;
pub(crate) mod row;
pub(crate) mod search;
pub(crate) mod selection;

use std::cell::Cell;
use std::cell::OnceCell;
use std::cell::RefCell;

use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

pub(crate) use self::avatar::Avatar;
pub(crate) use self::mini_thumbnail::MiniThumbnail;
pub(crate) use self::row::Row;
pub(crate) use self::search::ItemRow as SearchItemRow;
pub(crate) use self::search::Row as SearchRow;
pub(crate) use self::search::Search;
pub(crate) use self::search::Section as SearchSection;
pub(crate) use self::search::SectionRow as SearchSectionRow;
pub(crate) use self::search::SectionType as SearchSectionType;
pub(crate) use self::selection::Selection;
use crate::model;
use crate::ui;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/mod.ui")]
    pub(crate) struct Sidebar {
        pub(super) compact: Cell<bool>,
        pub(super) selected_chat: glib::WeakRef<model::Chat>,
        pub(super) marked_as_unread_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) session: glib::WeakRef<model::ClientStateSession>,
        pub(super) row_menu: OnceCell<gtk::PopoverMenu>,
        #[template_child]
        pub(super) snow: TemplateChild<ui::Snow>,
        #[template_child]
        pub(super) navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub(super) main_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) selection: TemplateChild<Selection>,
        #[template_child]
        pub(super) search: TemplateChild<Search>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "PaplSidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("sidebar.show-sessions", None, move |widget, _, _| {
                widget.show_sessions();
            });

            klass.install_action("sidebar.start-search", None, move |widget, _, _| {
                widget.begin_chats_search();
            });
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

            let item: model::ChatListItem =
                self.selection.selected_item().unwrap().downcast().unwrap();
            self.obj().set_selected_chat(item.chat().as_ref());
        }

        #[template_callback]
        fn close_search(&self) {
            self.navigation_view.pop_to_tag("chats");
        }
    }

    impl ObjectImpl for Sidebar {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("compact").build(),
                    glib::ParamSpecObject::builder::<model::Chat>("selected-chat")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<model::ClientStateSession>("session")
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

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "selected-chat" => obj.selected_chat().to_value(),
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.navigation_view.unparent();
        }
    }

    impl WidgetImpl for Sidebar {
        fn direction_changed(&self, previous_direction: gtk::TextDirection) {
            let obj = self.obj();

            if obj.direction() == previous_direction {
                return;
            }

            if let Some(menu) = self.row_menu.get() {
                menu.set_halign(if obj.direction() == gtk::TextDirection::Rtl {
                    gtk::Align::End
                } else {
                    gtk::Align::Start
                });
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget;
}

impl Sidebar {
    pub(crate) fn row_menu(&self) -> &gtk::PopoverMenu {
        self.imp().row_menu.get_or_init(|| {
            let menu =
                gtk::Builder::from_resource("/app/drey/paper-plane/ui/session/sidebar/row_menu.ui")
                    .object::<gtk::PopoverMenu>("menu")
                    .unwrap();

            menu.set_halign(if self.direction() == gtk::TextDirection::Rtl {
                gtk::Align::End
            } else {
                gtk::Align::Start
            });

            menu
        })
    }

    pub(crate) fn show_sessions(&self) {
        self.imp().navigation_view.push_by_tag("sessions");
    }

    pub(crate) fn begin_chats_search(&self) {
        let imp = self.imp();
        imp.search.reset();
        imp.search.grab_focus();
        imp.navigation_view.push_by_tag("search");
    }

    pub(crate) fn selected_chat(&self) -> Option<model::Chat> {
        self.imp().selected_chat.upgrade()
    }

    pub(crate) fn set_selected_chat(&self, selected_chat: Option<&model::Chat>) {
        if self.selected_chat().as_ref() == selected_chat {
            return;
        }

        let imp = self.imp();

        if let Some(handler_id) = imp.marked_as_unread_handler_id.take() {
            self.selected_chat().unwrap().disconnect(handler_id);
        }

        if let Some(chat) = selected_chat {
            let handler_id = chat.connect_notify_local(
                Some("is-marked-as-unread"),
                clone!(@weak self as obj => move |chat, _| {
                    if chat.is_marked_as_unread() {
                        obj.set_selected_chat(None);
                    }
                }),
            );
            imp.marked_as_unread_handler_id.replace(Some(handler_id));

            let item = chat.session_().main_chat_list().find_chat_item(chat.id());
            imp.selection.set_selected_item(item.map(|i| i.upcast()));

            if chat.is_marked_as_unread() {
                utils::spawn(clone!(@weak chat => async move {
                    if let Err(e) = chat.mark_as_read().await {
                        log::warn!("Error on toggling chat's unread state: {e:?}");
                    }
                }));
            }
        } else {
            imp.selection.set_selected_item(None);
        }

        imp.selected_chat.set(selected_chat);

        self.activate_action("navigation.push", Some(&"content".to_variant()))
            .unwrap();

        self.notify("selected-chat");
    }

    pub(crate) fn set_session(&self, session: Option<&model::ClientStateSession>) {
        if self.session().as_ref() == session {
            return;
        }

        let imp = self.imp();

        if let Some(session) = session {
            imp.selection
                .set_model(Some(session.main_chat_list().clone().upcast()));
        }

        imp.session.set(session);
        self.notify("session");
    }

    pub(crate) fn session(&self) -> Option<model::ClientStateSession> {
        self.imp().session.upgrade()
    }
}
