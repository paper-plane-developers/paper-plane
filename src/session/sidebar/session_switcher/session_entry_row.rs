use super::avatar_with_selection::AvatarWithSelection;

use glib::closure;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::tdlib::User;
use crate::{expressions, Session};

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/session-entry-row.ui")]
    pub(crate) struct SessionEntryRow {
        pub(super) session: RefCell<Option<Session>>,
        #[template_child]
        pub(super) account_avatar: TemplateChild<AvatarWithSelection>,
        #[template_child]
        pub(super) center_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) display_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) username_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) unread_count_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SessionEntryRow {
        const NAME: &'static str = "SessionEntryRow";
        type Type = super::SessionEntryRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            AvatarWithSelection::static_type();
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SessionEntryRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<Session>("session")
                        .construct_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("hint")
                        .write_only()
                        .construct_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "session" => obj.set_session(value.get().unwrap()),
                "hint" => obj.set_hint(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "session" => obj.session().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_expressions();
        }

        fn dispose(&self) {
            self.account_avatar.unparent();
            self.center_box.unparent();
            self.unread_count_label.unparent();
        }
    }

    impl WidgetImpl for SessionEntryRow {}
}

glib::wrapper! {
    pub(crate) struct SessionEntryRow(ObjectSubclass<imp::SessionEntryRow>)
        @extends gtk::Widget, @implements gtk::Accessible;
}

impl SessionEntryRow {
    pub(crate) fn new(session: &Session) -> Self {
        glib::Object::builder().property("session", session).build()
    }

    fn setup_expressions(&self) {
        let imp = self.imp();
        let me_expression =
            SessionEntryRow::this_expression("session").chain_property::<Session>("me");

        // Bind the name
        expressions::user_display_name(&me_expression).bind(
            &*imp.display_name_label,
            "label",
            Some(self),
        );

        // Bind the username
        let username_expression = me_expression.chain_property::<User>("username");
        username_expression
            .chain_closure::<String>(closure!(|_: SessionEntryRow, username: String| {
                format!("@{username}")
            }))
            .bind(&*imp.username_label, "label", Some(self));
        username_expression
            .chain_closure::<bool>(closure!(|_: SessionEntryRow, username: String| {
                !username.is_empty()
            }))
            .bind(&*imp.username_label, "visible", Some(self));
    }

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

    pub(crate) fn set_hint(&self, hinted: bool) {
        let imp = self.imp();
        imp.account_avatar.set_selected(hinted);
        imp.display_name_label
            .set_css_classes(if hinted { &["bold"] } else { &[] });
    }
}
