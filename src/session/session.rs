use glib::SyncSender;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::collections::HashMap;
use tdgrand::enums::Update;
use tdgrand::functions;
use tdgrand::types::File;

use crate::session::{Chat, ChatList, Content, Sidebar, UserList};
use crate::RUNTIME;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/session.ui")]
    pub struct Session {
        pub client_id: Cell<i32>,
        pub chat_list: OnceCell<ChatList>,
        pub user_list: UserList,
        pub selected_chat: RefCell<Option<Chat>>,
        pub downloading_files: RefCell<HashMap<i32, Vec<SyncSender<File>>>>,
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

            klass.install_action("session.log-out", None, move |widget, _, _| {
                widget.log_out();
            });
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
                "client-id" => obj.client_id().to_value(),
                "chat-list" => self.chat_list.get_or_init(|| ChatList::new(obj)).to_value(),
                "user-list" => self.user_list.to_value(),
                "selected-chat" => obj.selected_chat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

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
        glib::Object::new(&[("client-id", &client_id)]).expect("Failed to create Session")
    }

    pub fn handle_update(&self, update: Update) {
        let self_ = imp::Session::from_instance(self);

        match update {
            Update::NewMessage(_)
            | Update::MessageSendSucceeded(_)
            | Update::MessageContent(_)
            | Update::NewChat(_)
            | Update::ChatTitle(_)
            | Update::ChatPhoto(_)
            | Update::ChatLastMessage(_)
            | Update::ChatPosition(_)
            | Update::ChatReadInbox(_)
            | Update::ChatDraftMessage(_)
            | Update::DeleteMessages(_) => {
                self.chat_list().handle_update(update);
            }
            Update::User(_) => {
                self_.user_list.handle_update(update);
            }
            Update::File(update) => {
                self.handle_file_update(update.file);
            }
            _ => {}
        }
    }

    pub fn download_file(&self, file_id: i32, sender: SyncSender<File>) {
        let self_ = imp::Session::from_instance(self);

        let mut downloading_files = self_.downloading_files.borrow_mut();
        match downloading_files.get_mut(&file_id) {
            Some(senders) => {
                senders.push(sender);
            }
            None => {
                downloading_files.insert(file_id, vec![sender]);

                let client_id = self.client_id();
                RUNTIME.spawn(async move {
                    functions::DownloadFile::new()
                        .file_id(file_id)
                        .priority(5)
                        .send(client_id)
                        .await
                        .unwrap();
                });
            }
        }
    }

    fn handle_file_update(&self, file: File) {
        let self_ = imp::Session::from_instance(self);

        let mut downloading_files = self_.downloading_files.borrow_mut();
        if let Some(senders) = downloading_files.get(&file.id) {
            for sender in senders {
                sender.send(file.clone()).unwrap();
            }
        }

        if file.local.is_downloading_completed {
            downloading_files.remove(&file.id);
        }
    }

    fn log_out(&self) {
        let client_id = self.client_id();
        RUNTIME.spawn(async move {
            functions::LogOut::new().send(client_id).await.unwrap();
        });
    }

    pub fn client_id(&self) -> i32 {
        let self_ = imp::Session::from_instance(self);
        self_.client_id.get()
    }

    pub fn chat_list(&self) -> ChatList {
        self.property("chat-list").unwrap().get().unwrap()
    }

    pub fn user_list(&self) -> &UserList {
        let self_ = imp::Session::from_instance(self);
        &self_.user_list
    }

    fn selected_chat(&self) -> Option<Chat> {
        let self_ = imp::Session::from_instance(self);
        self_.selected_chat.borrow().clone()
    }

    fn set_selected_chat(&self, selected_chat: Option<Chat>) {
        if self.selected_chat() == selected_chat {
            return;
        }

        let self_ = imp::Session::from_instance(self);
        if selected_chat.is_some() {
            self_.leaflet.navigate(adw::NavigationDirection::Forward);
        } else {
            self_.leaflet.navigate(adw::NavigationDirection::Back);
        }

        self_.selected_chat.replace(selected_chat);
        self.notify("selected-chat");
    }

    fn fetch_chats(&self) {
        let self_ = imp::Session::from_instance(self);
        let client_id = self_.client_id.get();
        self.chat_list().fetch(client_id);
    }
}
