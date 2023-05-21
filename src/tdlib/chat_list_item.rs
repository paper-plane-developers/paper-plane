use std::cell::Cell;

use glib::WeakRef;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;
use tdlib::enums::ChatList as TdChatList;
use tdlib::functions;
use tdlib::types::ChatPosition as TdChatPosition;
use tdlib::types::Error as TdError;

use crate::tdlib::Chat;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ChatListItem {
        pub(super) chat: WeakRef<Chat>,
        pub(super) is_pinned: Cell<bool>,
        pub(super) chat_list_type: OnceCell<TdChatList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatListItem {
        const NAME: &'static str = "ChatListItem";
        type Type = super::ChatListItem;
    }

    impl ObjectImpl for ChatListItem {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<Chat>("chat")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-pinned")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "chat" => obj.chat().to_value(),
                "is-pinned" => obj.is_pinned().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatListItem(ObjectSubclass<imp::ChatListItem>);
}

impl ChatListItem {
    pub(crate) fn new(chat: &Chat, position: &TdChatPosition) -> ChatListItem {
        let obj: ChatListItem = glib::Object::new();
        let imp = obj.imp();

        imp.chat.set(Some(chat));
        imp.is_pinned.set(position.is_pinned);
        imp.chat_list_type.set(position.list.clone()).unwrap();

        obj
    }

    pub(crate) fn update(&self, position: &TdChatPosition) {
        self.set_is_pinned(position.is_pinned);
    }

    pub(crate) fn chat(&self) -> Chat {
        self.imp().chat.upgrade().unwrap()
    }

    pub(crate) fn is_pinned(&self) -> bool {
        self.imp().is_pinned.get()
    }

    fn set_is_pinned(&self, is_pinned: bool) {
        if self.is_pinned() == is_pinned {
            return;
        }
        self.imp().is_pinned.set(is_pinned);
        self.notify("is-pinned");
    }

    pub(crate) async fn toggle_is_pinned(&self) -> Result<(), TdError> {
        let chat = self.chat();
        functions::toggle_chat_is_pinned(
            self.imp().chat_list_type.get().unwrap().clone(),
            chat.id(),
            !self.is_pinned(),
            chat.session().client_id(),
        )
        .await
    }
}
