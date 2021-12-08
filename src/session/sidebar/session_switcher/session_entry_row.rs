use super::avatar_with_selection::AvatarWithSelection;

use gtk::{glib, prelude::*, subclass::prelude::*};

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
        pub bindings: RefCell<Vec<gtk::ExpressionWatch>>,
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
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "The session that this entry represents",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT_ONLY
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boolean(
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

    fn session(&self) -> Option<Session> {
        let self_ = imp::SessionEntryRow::from_instance(self);
        self_.session.borrow().clone()
    }

    pub fn set_session(&self, session: Option<Session>) {
        if self.session() == session {
            return;
        }

        let self_ = imp::SessionEntryRow::from_instance(self);
        let mut bindings = self_.bindings.borrow_mut();

        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        if let Some(ref session) = session {
            let me_expression =
                gtk::PropertyExpression::new(Session::static_type(), gtk::NONE_EXPRESSION, "me");

            let display_name_expression = gtk::ClosureExpression::new(
                |args| {
                    let maybe_me = args[1].get::<User>();
                    maybe_me
                        .ok()
                        .map(|user| {
                            let last_name = user.last_name();
                            if last_name.is_empty() {
                                user.first_name()
                            } else {
                                format!("{} {}", user.first_name(), last_name)
                            }
                        })
                        .unwrap_or_default()
                },
                &[me_expression.clone().upcast()],
            );
            let display_name_binding =
                display_name_expression.bind(&*self_.display_name_label, "label", Some(session));
            bindings.push(display_name_binding);

            let username_label_expression = gtk::ClosureExpression::new(
                |args| {
                    let maybe_me = args[1].get::<User>();
                    maybe_me
                        .ok()
                        .as_ref()
                        .map(User::username)
                        .filter(|username| !username.is_empty())
                        .map(|username| format!("@{}", username))
                        .unwrap_or_default()
                },
                &[me_expression.upcast()],
            );
            let username_label_binding =
                username_label_expression.bind(&*self_.username_label, "label", Some(session));
            bindings.push(username_label_binding);

            let username_visibility_expression = gtk::ClosureExpression::new(
                |args| {
                    let label = args[1].get::<&str>().unwrap();
                    !label.is_empty()
                },
                &[username_label_expression.upcast()],
            );
            let username_visibility_binding = username_visibility_expression.bind(
                &*self_.username_label,
                "visible",
                Some(session),
            );
            bindings.push(username_visibility_binding);
        }

        self_.session.replace(session);

        self.notify("session");
    }

    pub fn set_hint(&self, hinted: bool) {
        let self_ = imp::SessionEntryRow::from_instance(self);

        self_.account_avatar.set_selected(hinted);
        self_
            .display_name_label
            .set_css_classes(if hinted { &["bold"] } else { &[] });
    }
}
