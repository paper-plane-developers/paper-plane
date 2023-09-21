use std::cell::Cell;
use std::cell::OnceCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;

#[derive(Debug, Default, Clone, Copy, PartialEq, glib::Enum)]
#[enum_type(name = "SecretChatState")]
pub(crate) enum SecretChatState {
    #[default]
    Pending,
    Ready,
    Closed,
}

impl From<&tdlib::enums::SecretChatState> for SecretChatState {
    fn from(state: &tdlib::enums::SecretChatState) -> Self {
        use tdlib::enums::SecretChatState::*;

        match state {
            Pending => Self::Pending,
            Ready => Self::Ready,
            Closed => Self::Closed,
        }
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::SecretChat)]
    pub(crate) struct SecretChat {
        #[property(get, set, construct_only)]
        pub(super) user: glib::WeakRef<model::User>,
        #[property(get, set, construct_only)]
        pub(super) id: OnceCell<i32>,
        #[property(get, builder(SecretChatState::default()))]
        pub(super) state: Cell<SecretChatState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SecretChat {
        const NAME: &'static str = "SecretChat";
        type Type = super::SecretChat;
    }

    impl ObjectImpl for SecretChat {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub(crate) struct SecretChat(ObjectSubclass<imp::SecretChat>);
}

impl SecretChat {
    pub(crate) fn new(user: model::User, td_secret_chat: tdlib::types::SecretChat) -> Self {
        let obj: Self = glib::Object::builder()
            .property("user", user)
            .property("id", td_secret_chat.id)
            .build();

        obj.imp()
            .state
            .set(SecretChatState::from(&td_secret_chat.state));

        obj
    }

    pub(crate) fn user_(&self) -> model::User {
        self.user().unwrap()
    }

    fn set_state(&self, state: SecretChatState) {
        if self.state() == state {
            return;
        }
        self.imp().state.set(state);
        self.notify_state();
    }

    pub(crate) fn update(&self, td_secret_chat: tdlib::types::SecretChat) {
        self.set_state(SecretChatState::from(&td_secret_chat.state));
    }
}
