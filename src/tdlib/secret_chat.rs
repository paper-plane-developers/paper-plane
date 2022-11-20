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
                    glib::ParamSpecInt::builder("id").read_only().build(),
                    glib::ParamSpecObject::builder::<User>("user")
                        .read_only()
                        .build(),
                    glib::ParamSpecEnum::builder("state", SecretChatState::default())
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

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
        let secret_chat: SecretChat = glib::Object::builder().build();
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
