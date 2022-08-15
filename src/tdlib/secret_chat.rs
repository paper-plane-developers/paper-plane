use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::SecretChatState as TdSecretChatState;
use tdlib::types::SecretChat as TdSecretChat;

use crate::tdlib::User;

#[derive(Debug, Clone, Copy, PartialEq, glib::Enum)]
#[enum_type(name = "SecretChatState")]
pub(crate) enum SecretChatState {
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
    pub(crate) fn from_td_object(state: &TdSecretChatState) -> Self {
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
    pub(crate) struct SecretChat {
        pub(super) id: Cell<i32>,
        pub(super) user: OnceCell<User>,
        pub(super) state: Cell<SecretChatState>,
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
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecObject::new(
                        "user",
                        "User",
                        "The interlocutor in this chat",
                        User::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecEnum::new(
                        "state",
                        "State",
                        "The state of this secret chat",
                        SecretChatState::static_type(),
                        SecretChatState::default() as i32,
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
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
    pub(crate) struct SecretChat(ObjectSubclass<imp::SecretChat>);
}

impl SecretChat {
    pub(crate) fn from_td_object(td_secret_chat: TdSecretChat, user: User) -> Self {
        let secret_chat: SecretChat = glib::Object::new(&[]).expect("Failed to create SecretChat");
        let imp = secret_chat.imp();

        let state = SecretChatState::from_td_object(&td_secret_chat.state);

        imp.id.set(td_secret_chat.id);
        imp.user.set(user).unwrap();
        imp.state.set(state);

        secret_chat
    }

    pub(crate) fn update(&self, td_secret_chat: TdSecretChat) {
        self.set_state(SecretChatState::from_td_object(&td_secret_chat.state));
    }

    pub(crate) fn id(&self) -> i32 {
        self.imp().id.get()
    }

    pub(crate) fn user(&self) -> &User {
        self.imp().user.get().unwrap()
    }

    pub(crate) fn state(&self) -> SecretChatState {
        self.imp().state.get()
    }

    fn set_state(&self, state: SecretChatState) {
        if self.state() == state {
            return;
        }
        self.imp().state.set(state);
        self.notify("state");
    }
}
