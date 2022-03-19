use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::{SecretChatState as TdSecretChatState, Update};
use tdlib::types::SecretChat as TdSecretChat;

use crate::session::User;

#[derive(Debug, Clone, Copy, PartialEq, glib::Enum)]
#[enum_type(name = "SecretChatState")]
pub enum SecretChatState {
    Pending,
    Ready,
    Closed,
}

impl Default for SecretChatState {
    fn default() -> Self {
        Self::Pending
    }
}

impl SecretChatState {
    pub fn from_td_object(state: &TdSecretChatState) -> Self {
        match state {
            TdSecretChatState::Pending => Self::Pending,
            TdSecretChatState::Ready => Self::Ready,
            TdSecretChatState::Closed => Self::Closed,
        }
    }
}

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub struct SecretChat {
        pub id: Cell<i32>,
        pub user: OnceCell<User>,
        pub state: Cell<SecretChatState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SecretChat {
        const NAME: &'static str = "SecretChat";
        type Type = super::SecretChat;
    }

    impl ObjectImpl for SecretChat {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt::new(
                        "id",
                        "Id",
                        "The id of this secret chat",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "user",
                        "User",
                        "The interlocutor in this chat",
                        User::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecEnum::new(
                        "state",
                        "State",
                        "The state of this secret chat",
                        SecretChatState::static_type(),
                        SecretChatState::default() as i32,
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "id" => self.id.set(value.get().unwrap()),
                "user" => self.user.set(value.get().unwrap()).unwrap(),
                "state" => obj.set_state(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => obj.id().to_value(),
                "user" => obj.id().to_value(),
                "state" => obj.state().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SecretChat(ObjectSubclass<imp::SecretChat>);
}

impl SecretChat {
    pub fn from_td_object(secret_chat: &TdSecretChat, user: &User) -> Self {
        let state = SecretChatState::from_td_object(&secret_chat.state);
        glib::Object::new(&[("id", &secret_chat.id), ("user", user), ("state", &state)])
            .expect("Failed to create SecretChat")
    }

    pub fn handle_update(&self, update: &Update) {
        if let Update::SecretChat(data) = update {
            self.set_state(SecretChatState::from_td_object(&data.secret_chat.state));
        }
    }

    pub fn id(&self) -> i32 {
        self.imp().id.get()
    }

    pub fn user(&self) -> &User {
        self.imp().user.get().unwrap()
    }

    pub fn state(&self) -> SecretChatState {
        self.imp().state.get()
    }

    pub fn set_state(&self, state: SecretChatState) {
        if self.state() == state {
            return;
        }
        self.imp().state.set(state);
        self.notify("state");
    }
}
