use std::cell::Cell;
use std::cell::OnceCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::ChatListItem)]
    pub(crate) struct ChatListItem {
        pub(super) chat_list_type: OnceCell<tdlib::enums::ChatList>,
        #[property(get, set, construct_only)]
        pub(super) chat: glib::WeakRef<model::Chat>,
        #[property(get)]
        pub(super) is_pinned: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatListItem {
        const NAME: &'static str = "ChatListItem";
        type Type = super::ChatListItem;
    }

    impl ObjectImpl for ChatListItem {
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
    pub(crate) struct ChatListItem(ObjectSubclass<imp::ChatListItem>);
}

impl ChatListItem {
    pub(crate) fn new(chat: &model::Chat, position: &tdlib::types::ChatPosition) -> ChatListItem {
        let obj: Self = glib::Object::builder().property("chat", chat).build();

        let imp = obj.imp();
        imp.chat_list_type.set(position.list.clone()).unwrap();
        imp.is_pinned.set(position.is_pinned);

        obj
    }

    pub(crate) fn chat_(&self) -> model::Chat {
        self.chat().unwrap()
    }

    fn set_is_pinned(&self, is_pinned: bool) {
        if self.is_pinned() == is_pinned {
            return;
        }
        self.imp().is_pinned.set(is_pinned);
        self.notify_is_pinned();
    }

    pub(crate) async fn toggle_is_pinned(&self) -> Result<(), tdlib::types::Error> {
        let chat = self.chat_();

        tdlib::functions::toggle_chat_is_pinned(
            self.imp().chat_list_type.get().unwrap().clone(),
            chat.id(),
            !self.is_pinned(),
            chat.session_().client_().id(),
        )
        .await
    }

    pub(crate) fn update(&self, position: &tdlib::types::ChatPosition) {
        self.set_is_pinned(position.is_pinned);
    }
}
