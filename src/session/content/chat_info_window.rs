use adw::prelude::*;
use gettextrs::gettext;
use glib::{clone, closure};
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::ChatMembers::ChatMembers as TdChatMembers;
use tdlib::enums::User::User as TdUser;
use tdlib::enums::{ChatMemberStatus, MessageSender, UserType};
use tdlib::functions;
use tdlib::types::{BasicGroupFullInfo, ChatMember, ChatMembers, SupergroupFullInfo};

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
        pub(super) subtitle_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) info_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub(super) members_page: TemplateChild<adw::ViewStackPage>,
        #[template_child]
        pub(super) members_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatInfoWindow {
        const NAME: &'static str = "ContentChatInfoWindow";
        type Type = super::ChatInfoWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatInfoWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<Chat>("chat")
                    .construct_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "chat" => self.chat.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_window();
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
        glib::Object::builder()
            .property("transient-for", parent_window)
            .property("chat", chat)
            .build()
    }

    fn setup_window(&self) {
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
            ChatType::Secret(secret) => {
                self.setup_user_info(secret.user());
            }
        }
    }

    fn setup_user_info(&self, user: &User) {
        let imp = self.imp();

        // Online status or bot label
        if let UserType::Bot(_) = user.type_().0 {
            imp.subtitle_label.set_text(Some(&gettext("bot")));
        } else {
            User::this_expression("status")
                .chain_closure::<String>(closure!(
                    |_: Option<glib::Object>, status: BoxedUserStatus| {
                        strings::user_status(&status.0)
                    }
                ))
                .bind(&*imp.subtitle_label, "text", Some(user));
        }

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
            .bind(&*imp.subtitle_label, "text", Some(basic_group));

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

        imp.members_page.set_visible(true);
        spawn(clone!(@weak self as obj => async move {
            obj.append_members(basic_group_full_info.members).await;
        }));

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
            .bind(&*imp.subtitle_label, "text", Some(supergroup));

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
                    obj.setup_supergroup_full_info(supergroup_id, full_info);
                }
                Err(e) => {
                    log::warn!("Failed to get supergroup full info: {e:?}");
                }
            }
        }));
    }

    fn setup_supergroup_full_info(
        &self,
        supergroup_id: i64,
        supergroup_full_info: SupergroupFullInfo,
    ) {
        let client_id = self.chat().unwrap().session().client_id();
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

        if supergroup_full_info.can_get_members {
            imp.members_page.set_visible(true);
            spawn(clone!(@weak self as obj => async move {
                let limit = 200;
                let mut offset = 0;
                while let Ok(TdChatMembers(ChatMembers {members, total_count})) = functions::get_supergroup_members(
                    supergroup_id,
                    None,
                    offset,
                    limit,
                    client_id,
                ).await
                {
                    if offset > total_count {
                        break;
                    }

                    obj.append_members(members).await;

                    offset += limit;
                }
            }));
        }

        self.update_info_list_visibility();
    }

    async fn append_members(&self, members: Vec<ChatMember>) {
        let session = self.chat().unwrap().session();
        let client_id = session.client_id();

        let members_list = &self.imp().members_list;

        for member in members {
            if let MessageSender::User(user) = member.member_id {
                if let Ok(TdUser(user)) = functions::get_user(user.user_id, client_id).await {
                    let user_row = adw::ActionRow::new();
                    user_row.set_title_lines(1);
                    user_row.set_subtitle_lines(1);

                    let user = User::from_td_object(user, &session);

                    let user_expression = gtk::ObjectExpression::new(&user);
                    let name_expression = expressions::user_display_name(&user_expression);
                    name_expression.bind(&user_row, "title", Some(&user));

                    User::this_expression("status")
                        .chain_closure::<String>(closure!(
                            |_: Option<glib::Object>, status: BoxedUserStatus| {
                                strings::user_status(&status.0)
                            }
                        ))
                        .bind(&user_row, "subtitle", Some(&user));

                    if let UserType::Bot(_) = user.type_().0 {
                        user_row.set_subtitle(&gettext("bot"));
                    } else {
                        User::this_expression("status")
                            .chain_closure::<String>(closure!(
                                |_: Option<glib::Object>, status: BoxedUserStatus| {
                                    strings::user_status(&status.0)
                                }
                            ))
                            .bind(&user_row, "subtitle", Some(&user));
                    };

                    let avatar = crate::session::components::Avatar::new();

                    avatar.set_item(Some(user.upcast()));
                    avatar.set_size(32);
                    user_row.add_prefix(&avatar);

                    let status = match member.status {
                        ChatMemberStatus::Creator(owner) => {
                            let title = if owner.custom_title.is_empty() {
                                gettext("Owner")
                            } else {
                                owner.custom_title
                            };
                            Some(title)
                        }
                        ChatMemberStatus::Administrator(admin) => {
                            let title = if admin.custom_title.is_empty() {
                                gettext("Admin")
                            } else {
                                admin.custom_title
                            };
                            Some(title)
                        }
                        _ => None,
                    };

                    if let Some(text) = status {
                        let owner_label = gtk::Label::new(Some(&text));
                        owner_label.set_yalign(0.2);
                        owner_label.set_css_classes(&["caption", "accent"]);
                        user_row.add_suffix(&owner_label);
                    }

                    members_list.append(&user_row);
                }
            }
        }
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
