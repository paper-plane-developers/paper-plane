use adw::prelude::*;
use gettextrs::gettext;
use glib::{clone, closure};
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::functions;
use tdlib::types::{BasicGroupFullInfo, SupergroupFullInfo};

use crate::i18n::ngettext_f;
use crate::tdlib::{BasicGroup, BoxedUserStatus, Chat, ChatType, Supergroup, User};
use crate::utils::spawn;
use crate::{expressions, strings};

mod imp {
    use super::*;
    use adw::subclass::prelude::AdwWindowImpl;
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-info-window.ui")]
    pub(crate) struct ChatInfoWindow {
        pub(super) chat: OnceCell<Chat>,
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub(super) name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) subtitle_label: TemplateChild<gtk::Label>,
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
            ChatType::BasicGroup(basic_group) => {
                self.setup_basic_group_info(basic_group);
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

        // Online status
        User::this_expression("status")
            .chain_closure::<String>(closure!(
                |_: Option<glib::Object>, status: BoxedUserStatus| {
                    strings::user_status(&status.0)
                }
            ))
            .bind(&*imp.subtitle_label, "label", Some(user));

        // Phone number
        if !user.phone_number().is_empty() {
            let row = adw::ActionRow::builder()
                .title(&format!("+{}", &user.phone_number()))
                .subtitle(&gettext("Mobile"))
                .build();
            self.make_row_copyable(&row);
            imp.info_list.append(&row);
        }

        // Username
        if !user.username().is_empty() {
            let row = adw::ActionRow::builder()
                .title(&format!("@{}", &user.username()))
                .subtitle(&gettext("Username"))
                .build();
            self.make_row_copyable(&row);
            imp.info_list.append(&row);
        }

        self.update_info_list_visibility();
    }

    fn setup_basic_group_info(&self, basic_group: &BasicGroup) {
        let client_id = self.chat().unwrap().session().client_id();
        let basic_group_id = basic_group.id();
        let imp = self.imp();

        // Members number
        BasicGroup::this_expression("member-count")
            .chain_closure::<String>(closure!(|_: Option<glib::Object>, member_count: i32| {
                ngettext_f(
                    "{num} member",
                    "{num} members",
                    member_count as u32,
                    &[("num", &member_count.to_string())],
                )
            }))
            .bind(&*imp.subtitle_label, "label", Some(basic_group));

        self.update_info_list_visibility();

        // Full info
        spawn(clone!(@weak self as obj => async move {
            let result = functions::get_basic_group_full_info(basic_group_id, client_id).await;
            match result {
                Ok(tdlib::enums::BasicGroupFullInfo::BasicGroupFullInfo(full_info)) => {
                    obj.setup_basic_group_full_info(full_info);
                }
                Err(e) => {
                    log::warn!("Failed to get basic group full info: {e:?}");
                }
            }
        }));
    }

    fn setup_basic_group_full_info(&self, basic_group_full_info: BasicGroupFullInfo) {
        let imp = self.imp();

        // Description
        if !basic_group_full_info.description.is_empty() {
            let row = adw::ActionRow::builder()
                .title(&basic_group_full_info.description)
                .subtitle(&gettext("Description"))
                .build();
            self.make_row_copyable(&row);
            imp.info_list.append(&row);
        }

        self.update_info_list_visibility();
    }

    fn setup_supergroup_info(&self, supergroup: &Supergroup) {
        let client_id = self.chat().unwrap().session().client_id();
        let supergroup_id = supergroup.id();
        let imp = self.imp();

        // Members number
        Supergroup::this_expression("member-count")
            .chain_closure::<String>(closure!(|_: Option<glib::Object>, member_count: i32| {
                ngettext_f(
                    "{num} member",
                    "{num} members",
                    member_count as u32,
                    &[("num", &member_count.to_string())],
                )
            }))
            .bind(&*imp.subtitle_label, "label", Some(supergroup));

        // Link
        if !supergroup.username().is_empty() {
            let row = adw::ActionRow::builder()
                .title(&format!("https://t.me/{}", &supergroup.username()))
                .subtitle(&gettext("Link"))
                .build();
            self.make_row_copyable(&row);
            imp.info_list.append(&row);
        }

        self.update_info_list_visibility();

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
            self.make_row_copyable(&row);
            imp.info_list.append(&row);
        }

        self.update_info_list_visibility();
    }

    fn update_info_list_visibility(&self) {
        let info_list = &self.imp().info_list;
        info_list.set_visible(info_list.first_child().is_some());
    }

    fn make_row_copyable(&self, action_row: &adw::ActionRow) {
        action_row.set_activatable(true);
        action_row.connect_activated(clone!(@weak self as obj => move |action_row| {
            action_row.clipboard().set_text(&action_row.title());

            let toast = adw::Toast::new(&gettext("Copied to clipboard"));
            obj.imp().toast_overlay.add_toast(&toast);
        }));
    }

    pub(crate) fn chat(&self) -> Option<&Chat> {
        self.imp().chat.get()
    }
}
