use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::Session;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-search.ui")]
    pub(crate) struct Search {
        pub(super) session: RefCell<Option<Session>>,
        pub(super) compact: Cell<bool>,
        #[template_child]
        pub(super) content: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) search_entry: TemplateChild<gtk::SearchEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Search {
        const NAME: &'static str = "SidebarSearch";
        type Type = super::Search;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.install_action("sidebar-search.go-back", None, move |widget, _, _| {
                widget.emit_by_name::<()>("close", &[]);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Search {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("close", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "compact",
                        "Compact",
                        "Whether a compact view is used or not",
                        false,
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
                "session" => obj.set_session(value.get().unwrap()),
                "compact" => obj.set_compact(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "session" => obj.session().to_value(),
                "compact" => obj.compact().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.content.unparent();
        }
    }

    impl WidgetImpl for Search {
        fn grab_focus(&self, _widget: &Self::Type) -> bool {
            self.search_entry.grab_focus();
            true
        }
    }
}

glib::wrapper! {
    pub(crate) struct Search(ObjectSubclass<imp::Search>)
        @extends gtk::Widget;
}

impl Search {
    pub(crate) fn session(&self) -> Option<Session> {
        self.imp().session.borrow().clone()
    }

    pub(crate) fn set_session(&self, session: Option<Session>) {
        if self.session() == session {
            return;
        }
        self.imp().session.replace(session);
        self.notify("session");
    }

    pub(crate) fn compact(&self) -> bool {
        self.imp().compact.get()
    }

    pub(crate) fn set_compact(&self, compact: bool) {
        if self.compact() == compact {
            return;
        }
        self.imp().compact.set(compact);
        self.notify("compact");
    }
}
