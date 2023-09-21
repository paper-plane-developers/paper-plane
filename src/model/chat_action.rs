use std::cell::OnceCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::ChatAction)]
    pub(crate) struct ChatAction {
        #[property(get, set, construct_only)]
        pub(super) chat: glib::WeakRef<model::Chat>,
        #[property(get, set, construct_only)]
        pub(super) action_type: OnceCell<model::BoxedChatActionType>,
        #[property(get, set, construct_only)]
        pub(super) sender: OnceCell<model::MessageSender>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatAction {
        const NAME: &'static str = "ChatAction";
        type Type = super::ChatAction;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ChatAction {
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
    pub(crate) struct ChatAction(ObjectSubclass<imp::ChatAction>);
}

impl ChatAction {
    pub(crate) fn new(
        type_: tdlib::enums::ChatAction,
        sender: &tdlib::enums::MessageSender,
        chat: &model::Chat,
    ) -> Self {
        glib::Object::builder()
            .property("chat", chat)
            .property("action-type", model::BoxedChatActionType(type_))
            .property(
                "sender",
                model::MessageSender::new(&chat.session_(), sender),
            )
            .build()
    }

    pub(crate) fn chat_(&self) -> model::Chat {
        self.chat().unwrap()
    }
}
