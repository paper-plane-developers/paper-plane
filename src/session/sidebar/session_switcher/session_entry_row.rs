use super::avatar_with_selection::AvatarWithSelection;

use glib::closure;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::session::{Session, User};

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/session-entry-row.ui")]
    pub struct SessionEntryRow {
        pub session: RefCell<Option<Session>>,
        #[template_child]
        pub account_avatar: TemplateChild<AvatarWithSelection>,
        #[template_child]
        pub center_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub display_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub username_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub unread_count_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SessionEntryRow {
        const NAME: &'static str = "SessionEntryRow";
        type Type = super::SessionEntryRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            AvatarWithSelection::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SessionEntryRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "session",
                        "Session",
                        "The session that this entry represents",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT_ONLY
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "hint",
                        "Selection hint",
                        "The hint of the session that owns the account switcher which this entry belongs to",
                        false,
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "session" => {
                    let session_page = value.get().unwrap();
                    obj.set_session(session_page);
                }
                "hint" => obj.set_hint(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_expressions();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.account_avatar.unparent();
            self.center_box.unparent();
            self.unread_count_label.unparent();
        }
    }

    impl WidgetImpl for SessionEntryRow {}
}

glib::wrapper! {
    pub struct SessionEntryRow(ObjectSubclass<imp::SessionEntryRow>)
        @extends gtk::Widget, @implements gtk::Accessible;
}

impl SessionEntryRow {
    pub fn new(session: &Session) -> Self {
        glib::Object::new(&[("session", session)]).expect("Failed to create SessionEntryRow")
    }

    fn setup_expressions(&self) {
        let imp = self.imp();
        let me_expression =
            SessionEntryRow::this_expression("session").chain_property::<Session>("me");

        // Bind the name
        User::full_name_expression(&me_expression).bind(
            &*imp.display_name_label,
            "label",
            Some(self),
        );

        // Bind the username
        let username_expression = me_expression.chain_property::<User>("username");
        username_expression
            .chain_closure::<String>(closure!(|_: SessionEntryRow, username: String| {
                format!("@{}", username)
            }))
            .bind(&*imp.username_label, "label", Some(self));
        username_expression
            .chain_closure::<bool>(closure!(|_: SessionEntryRow, username: String| {
                !username.is_empty()
            }))
            .bind(&*imp.username_label, "visible", Some(self));
    }

    pub fn session(&self) -> Option<Session> {
        self.imp().session.borrow().clone()
    }

    pub fn set_session(&self, session: Option<Session>) {
        if self.session() == session {
            return;
        }
        self.imp().session.replace(session);
        self.notify("session");
    }

    pub fn set_hint(&self, hinted: bool) {
        let imp = self.imp();
        imp.account_avatar.set_selected(hinted);
        imp.display_name_label
            .set_css_classes(if hinted { &["bold"] } else { &[] });
    }
}
