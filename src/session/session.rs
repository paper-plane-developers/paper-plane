use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tdgrand::enums::Update;

use crate::session::{Chat, ChatList, Content, Sidebar, UserList};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/session.ui")]
    pub struct Session {
        pub client_id: Cell<i32>,
        pub chat_list: ChatList,
        pub user_list: UserList,
        pub selected_chat: RefCell<Option<Chat>>,
        #[template_child]
        pub leaflet: TemplateChild<adw::Leaflet>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Session {
        const NAME: &'static str = "Session";
        type Type = super::Session;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Sidebar::static_type();
            Content::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Session {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int(
                        "client-id",
                        "Client Id",
                        "The client id",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "chat-list",
                        "Chat List",
                        "A list of chats",
                        ChatList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "user-list",
                        "User List",
                        "The list of users of this session",
                        ChatList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-chat",
                        "Selected Chat",
                        "The selected chat in this sidebar",
                        Chat::static_type(),
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
                "client-id" => {
                    let client_id = value.get().unwrap();
                    self.client_id.set(client_id);
                }
                "selected-chat" => {
                    let selected_chat = value.get().unwrap();
                    obj.set_selected_chat(selected_chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "client-id" => self.client_id.get().to_value(),
                "chat-list" => self.chat_list.to_value(),
                "user-list" => self.user_list.to_value(),
                "selected-chat" => obj.selected_chat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let session_expression = gtk::ConstantExpression::new(obj);
            session_expression.bind(&self.chat_list, "session", Some(&self.chat_list));

            obj.fetch_chats();
        }
    }

    impl WidgetImpl for Session {}
    impl BinImpl for Session {}
}

glib::wrapper! {
    pub struct Session(ObjectSubclass<imp::Session>)
        @extends gtk::Widget, adw::Bin;
}

impl Session {
    pub fn new(client_id: i32) -> Self {
        glib::Object::new(&[("client-id", &client_id)])
            .expect("Failed to create Session")
    }

    pub fn handle_update(&self, update: Update) {
        let priv_ = imp::Session::from_instance(self);

        match update {
            Update::NewMessage(_) | Update::NewChat(_) | Update::ChatTitle(_) |
                Update::ChatLastMessage(_) | Update::ChatPosition(_) |
                Update::ChatReadInbox(_) => {
                    priv_.chat_list.handle_update(update);
            },
            Update::User(_) => {
                priv_.user_list.handle_update(update);
            }
            _ => (),
        }
    }

    pub fn chat_list(&self) -> &ChatList {
        let priv_ = imp::Session::from_instance(self);
        &priv_.chat_list
    }

    pub fn user_list(&self) -> &UserList {
        let priv_ = imp::Session::from_instance(self);
        &priv_.user_list
    }

    fn selected_chat(&self) -> Option<Chat> {
        let priv_ = imp::Session::from_instance(self);
        priv_.selected_chat.borrow().clone()
    }

    fn set_selected_chat(&self, selected_chat: Option<Chat>) {
        if self.selected_chat() == selected_chat {
            return;
        }

        let priv_ = imp::Session::from_instance(self);
        if selected_chat.is_some() {
            priv_.leaflet.navigate(adw::NavigationDirection::Forward);
        } else {
            priv_.leaflet.navigate(adw::NavigationDirection::Back);
        }

        priv_.selected_chat.replace(selected_chat);
        self.notify("selected-chat");
    }

    fn fetch_chats(&self) {
        let priv_ = imp::Session::from_instance(self);
        let client_id = priv_.client_id.get();
        priv_.chat_list.fetch(client_id);
    }
}
