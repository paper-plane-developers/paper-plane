use glib::closure;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::expressions;
use crate::session::User;

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwWindowImpl;
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-info-dialog.ui")]
    pub(crate) struct ChatInfoDialog {
        pub(super) user: OnceCell<User>,
        #[template_child]
        pub(super) name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) mobile_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) username_row: TemplateChild<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatInfoDialog {
        const NAME: &'static str = "ContentChatInfoDialog";
        type Type = super::ChatInfoDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatInfoDialog {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "user",
                    "User",
                    "The user displayed by this dialog",
                    User::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "user" => self.user.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "user" => obj.user().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_expressions();
        }
    }

    impl WidgetImpl for ChatInfoDialog {}
    impl WindowImpl for ChatInfoDialog {}
    impl AdwWindowImpl for ChatInfoDialog {}
}

glib::wrapper! {
    pub(crate) struct ChatInfoDialog(ObjectSubclass<imp::ChatInfoDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl ChatInfoDialog {
    pub(crate) fn new(parent_window: &Option<gtk::Window>, user: &User) -> Self {
        glib::Object::new(&[("transient-for", parent_window), ("user", user)])
            .expect("Failed to create ChatInfoDialog")
    }

    fn setup_expressions(&self) {
        let imp = self.imp();
        let user_expression = Self::this_expression("user");

        // Bind the name
        expressions::user_full_name(&user_expression).bind(&*imp.name_label, "label", Some(self));

        // Bind the phone number
        let phone_number_expression = user_expression.chain_property::<User>("phone-number");
        phone_number_expression
            .chain_closure::<String>(closure!(|_: ChatInfoDialog, phone_number: String| {
                format!("+{}", phone_number)
            }))
            .bind(&*imp.mobile_row, "title", Some(self));
        phone_number_expression
            .chain_closure::<bool>(closure!(|_: ChatInfoDialog, phone_number: String| {
                !phone_number.is_empty()
            }))
            .bind(&*imp.mobile_row, "visible", Some(self));

        // Bind the username
        let username_expression = user_expression.chain_property::<User>("username");
        username_expression
            .chain_closure::<String>(closure!(|_: ChatInfoDialog, username: String| {
                format!("@{}", username)
            }))
            .bind(&*imp.username_row, "title", Some(self));
        username_expression
            .chain_closure::<bool>(closure!(|_: ChatInfoDialog, username: String| {
                !username.is_empty()
            }))
            .bind(&*imp.username_row, "visible", Some(self));
    }

    pub(crate) fn user(&self) -> Option<&User> {
        self.imp().user.get()
    }
}
