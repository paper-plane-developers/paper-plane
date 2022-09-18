use gettextrs::gettext;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::functions;
use tdlib::types::SupergroupFullInfo;

use crate::expressions;
use crate::tdlib::{Chat, ChatType, Supergroup, User};
use crate::utils::spawn;

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
            ChatType::Supergroup(supergroup) => {
                self.setup_supergroup_info(supergroup);
            }
            _ => {
                imp.info_list.set_visible(false);
            }
        }
    }

    fn setup_user_info(&self, user: &User) {
        let imp = self.imp();

        // Phone number
        if !user.phone_number().is_empty() {
            let row = adw::ActionRow::builder()
                .title(&format!("+{}", &user.phone_number()))
                .subtitle(&gettext("Mobile"))
                .build();
            imp.info_list.append(&row);
        }

        // Username
        if !user.username().is_empty() {
            let row = adw::ActionRow::builder()
                .title(&format!("@{}", &user.username()))
                .subtitle(&gettext("Username"))
                .build();
            imp.info_list.append(&row);
        }
    }

    fn setup_supergroup_info(&self, supergroup: &Supergroup) {
        let client_id = self.chat().unwrap().session().client_id();
        let supergroup_id = supergroup.id();
        let imp = self.imp();

        // Link
        if !supergroup.username().is_empty() {
            let row = adw::ActionRow::builder()
                .title(&format!("https://t.me/{}", &supergroup.username()))
                .subtitle(&gettext("Link"))
                .build();
            imp.info_list.append(&row);
        }

        // Full info
        spawn(clone!(@weak self as obj => async move {
            let result = functions::get_supergroup_full_info(supergroup_id, client_id).await;
            match result {
                Ok(tdlib::enums::SupergroupFullInfo::SupergroupFullInfo(full_info)) => {
                    obj.setup_supergroup_full_info(full_info);
                }
                Err(e) => {
                    log::warn!("Failed to get supergroup full info: {e:?}");
                }
            }
        }));
    }

    fn setup_supergroup_full_info(&self, supergroup_full_info: SupergroupFullInfo) {
        let imp = self.imp();

        // Description
        if !supergroup_full_info.description.is_empty() {
            let row = adw::ActionRow::builder()
                .title(&supergroup_full_info.description)
                .subtitle(&gettext("Description"))
                .build();
            imp.info_list.append(&row);
        }
    }

    pub(crate) fn chat(&self) -> Option<&Chat> {
        self.imp().chat.get()
    }
}
