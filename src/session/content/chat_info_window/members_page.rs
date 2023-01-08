use super::member_row::MemberRow;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::BasicGroupFullInfo::BasicGroupFullInfo as TdBasicGroupFullInfo;
use tdlib::enums::ChatMembers::ChatMembers as TdChatMembers;
use tdlib::enums::MessageSender;
use tdlib::enums::SupergroupFullInfo::SupergroupFullInfo as TdSupergroupFullInfo;
use tdlib::enums::User::User as TdUser;
use tdlib::functions;
use tdlib::types::{ChatMember as TdChatMember, ChatMembers};

use crate::tdlib::{BasicGroup, Chat, ChatMember, ChatType, Supergroup, User};
use crate::utils::spawn;

mod imp {
    use super::*;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="ContentChatInfoMembers" parent="AdwBin">
        <property name="child">
          <object class="GtkScrolledWindow">
            <property name="child">
              <object class="AdwClampScrollable">
                <property name="maximum-size">440</property>
                <property name="tightening-threshold">200</property>
                <property name="child">
                  <object class="GtkListView" id="members_list">
                    <property name="model">
                      <object class="GtkNoSelection" id="selection"/>
                    </property>
                    <style>
                      <class name="navigation-sidebar"/>
                    </style>
                  </object>
                </property>
              </object>
            </property>
          </object>
        </property>
      </template>
    </interface>
    "#)]
    pub(crate) struct ChatInfoMembers {
        pub(super) loading: Cell<bool>,
        pub(super) chat: OnceCell<Chat>,
        #[template_child]
        pub(super) members_list: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatInfoMembers {
        const NAME: &'static str = "ContentChatInfoMembers";
        type Type = super::ChatInfoMembers;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatInfoMembers {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<Chat>("chat")
                    // .construct_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "chat" => {
                    if let Some(chat) = value.get().unwrap() {
                        self.chat.set(chat).unwrap();
                        self.obj().setup_page();
                    }
                }
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
    }

    impl WidgetImpl for ChatInfoMembers {}
    impl BinImpl for ChatInfoMembers {}
}

glib::wrapper! {
    pub(crate) struct ChatInfoMembers(ObjectSubclass<imp::ChatInfoMembers>)
        @extends gtk::Widget;
}

impl ChatInfoMembers {
    fn setup_page(&self) {
        match self.chat().unwrap().type_() {
            ChatType::BasicGroup(basic_group) => {
                self.setup_basic_group_info(basic_group);
            }
            ChatType::Supergroup(supergroup) => {
                self.setup_supergroup_info(supergroup);
            }
            _ => {
                self.set_visible(false);
            }
        }
    }

    fn setup_basic_group_info(&self, basic_group: &BasicGroup) {
        let client_id = self.chat().unwrap().session().client_id();
        let basic_group_id = basic_group.id();

        spawn(clone!(@weak self as obj => async move {
            let result = functions::get_basic_group_full_info(basic_group_id, client_id).await;
            if let Ok(TdBasicGroupFullInfo(full_info)) = result {
                obj.append_members(full_info.members).await;
            }
        }));
    }

    fn setup_supergroup_info(&self, supergroup: &Supergroup) {
        let client_id = self.chat().unwrap().session().client_id();
        let supergroup_id = supergroup.id();

        spawn(clone!(@weak self as obj => async move {
            let imp = obj.imp();
            let result = functions::get_supergroup_full_info(supergroup_id, client_id).await;
            if let Ok(TdSupergroupFullInfo(full_info)) = result {
                if full_info.can_get_members {
                    imp.loading.set(true);
                    let result = functions::get_supergroup_members(
                                    supergroup_id,
                                    None,
                                    0,
                                    200,
                                    client_id,
                                ).await;
                    if let Ok(TdChatMembers(ChatMembers {members, total_count})) = result {
                        obj.append_members(members).await;

                        if total_count > 200 {
                            obj.imp().members_list.vadjustment().unwrap()
                            .connect_changed(clone!(@weak  obj => move |adj| {
                                obj.load_more_members(adj, supergroup_id);
                            }));
                        }
                    }
                    imp.loading.set(false);
                }
            }
        }));
    }

    fn load_more_members(&self, adj: &gtk::Adjustment, supergroup_id: i64) {
        let imp = self.imp();
        if imp.loading.get() {
            return;
        }
        imp.loading.set(true);

        if adj.value() > adj.page_size() * 2.0 || adj.upper() >= adj.page_size() * 2.0 {
            let offset = imp.members_list.model().unwrap().n_items() as i32;
            let limit = 200;
            let client_id = self.chat().unwrap().session().client_id();

            spawn(clone!(@weak self as obj => async move {
                let result = functions::get_supergroup_members(
                                supergroup_id,
                                None,
                                offset,
                                limit,
                                client_id,
                            ).await;
                if let Ok(TdChatMembers(ChatMembers {members, ..})) = result {
                    obj.append_members(members).await;
                    obj.imp().loading.set(false);
                } else {
                    log::error!("can't load members {result:?}");
                }
            }));
        }
    }

    async fn append_members(&self, members: Vec<TdChatMember>) {
        let members: Vec<_> = {
            let mut users: Vec<User> = vec![];

            let session = self.chat().unwrap().session();
            let client_id = session.client_id();

            for member in &members {
                let user = match member.member_id {
                    MessageSender::User(ref user) => {
                        let TdUser(user) =
                            functions::get_user(user.user_id, client_id).await.unwrap();
                        User::from_td_object(user, &session)
                    }
                    MessageSender::Chat(_) => unreachable!(),
                };
                users.push(user);
            }

            members
                .into_iter()
                .zip(users.into_iter())
                .map(|(member, user)| ChatMember::new(member, user))
                .collect()
        };

        let members_list = &self.imp().members_list;

        let selection_model: gtk::NoSelection = members_list.model().unwrap().downcast().unwrap();

        let model: gtk::gio::ListStore = if let Some(model) = selection_model.model() {
            model.downcast().unwrap()
        } else {
            let model = gtk::gio::ListStore::new(ChatMember::static_type());
            selection_model.set_model(Some(&model));
            model
        };

        model.extend_from_slice(&members);

        if members_list.factory().is_none() {
            let factory = gtk::SignalListItemFactory::new();

            factory.connect_setup(move |_, list_item| {
                list_item.set_property("child", MemberRow::new());
            });

            factory.connect_bind(move |_, list_item| {
                let list_item: &gtk::ListItem = list_item.downcast_ref().unwrap();

                let user_row: MemberRow = list_item.child().unwrap().downcast().unwrap();
                let member: ChatMember = list_item.item().unwrap().downcast().unwrap();

                user_row.bind_member(member);
            });

            members_list.set_factory(Some(&factory));
        }
    }

    pub(crate) fn chat(&self) -> Option<&Chat> {
        self.imp().chat.get()
    }
}
