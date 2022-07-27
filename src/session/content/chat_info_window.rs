use gettextrs::gettext;
use glib::closure;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::expressions;
use crate::tdlib::{Chat, ChatType, User};

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwWindowImpl;
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-info-window.ui")]
    pub(crate) struct ChatInfoWindow {
        pub(super) chat: OnceCell<Chat>,
        #[template_child]
        pub(super) name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) info_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatInfoWindow {
        const NAME: &'static str = "ContentChatInfoWindow";
        type Type = super::ChatInfoWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatInfoWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "chat",
                    "Chat",
                    "The chat displayed by this dialog",
                    Chat::static_type(),
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
                "chat" => self.chat.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_dialog();
        }
    }

    impl WidgetImpl for ChatInfoWindow {}
    impl WindowImpl for ChatInfoWindow {}
    impl AdwWindowImpl for ChatInfoWindow {}
}

glib::wrapper! {
    pub(crate) struct ChatInfoWindow(ObjectSubclass<imp::ChatInfoWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl ChatInfoWindow {
    pub(crate) fn new(parent_window: &Option<gtk::Window>, chat: &Chat) -> Self {
        glib::Object::new(&[("transient-for", parent_window), ("chat", chat)])
            .expect("Failed to create ChatInfoWindow")
    }

    fn setup_dialog(&self) {
        let imp = self.imp();
        let chat_expression = Self::this_expression("chat");

        // Bind the name
        expressions::chat_display_name(&chat_expression).bind(
            &*imp.name_label,
            "label",
            Some(self),
        );

        match self.chat().unwrap().type_() {
            ChatType::Private(user) => {
                self.setup_user_info(user);
            }
            _ => {
                imp.info_list.set_visible(false);
            }
        }
    }

    fn setup_user_info(&self, user: &User) {
        let imp = self.imp();

        // Phone number
        let mobile_row = adw::ActionRow::builder()
            .subtitle(&gettext("Mobile"))
            .icon_name("phone-oldschool-symbolic")
            .build();
        imp.info_list.append(&mobile_row);

        let phone_number_expression = User::this_expression("phone-number");
        phone_number_expression
            .chain_closure::<String>(closure!(|_: User, phone_number: String| {
                format!("+{}", phone_number)
            }))
            .bind(&mobile_row, "title", Some(user));
        phone_number_expression
            .chain_closure::<bool>(closure!(|_: User, phone_number: String| {
                !phone_number.is_empty()
            }))
            .bind(&mobile_row, "visible", Some(user));

        // Username
        let username_row = adw::ActionRow::builder()
            .subtitle(&gettext("Username"))
            .icon_name("user-info-symbolic")
            .build();
        imp.info_list.append(&username_row);

        let username_expression = User::this_expression("username");
        username_expression
            .chain_closure::<String>(closure!(|_: User, username: String| {
                format!("@{}", username)
            }))
            .bind(&username_row, "title", Some(user));
        username_expression
            .chain_closure::<bool>(closure!(|_: User, username: String| {
                !username.is_empty()
            }))
            .bind(&username_row, "visible", Some(user));
    }

    pub(crate) fn chat(&self) -> Option<&Chat> {
        self.imp().chat.get()
    }
}
