use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use std::mem;
use tdlib::{enums, types};

use crate::tdlib::{Chat, ChatAction};

mod imp {
    use super::*;

    use gtk::glib::WeakRef;
    use indexmap::IndexMap;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub(crate) struct ChatActionList {
        pub(super) list: RefCell<IndexMap<i64, ChatAction>>,
        pub(super) chat: WeakRef<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatActionList {
        const NAME: &'static str = "ChatActionList";
        type Type = super::ChatActionList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ChatActionList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "chat",
                    "Chat",
                    "The chat relative to this chat action list",
                    Chat::static_type(),
                    glib::ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for ChatActionList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            ChatAction::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|(_, obj)| obj.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatActionList(ObjectSubclass<imp::ChatActionList>)
        @implements gio::ListModel;
}

impl From<&Chat> for ChatActionList {
    fn from(chat: &Chat) -> Self {
        let chat_action_list: ChatActionList =
            glib::Object::new(&[]).expect("Failed to create ChatActionList");
        chat_action_list.imp().chat.set(Some(chat));
        chat_action_list
    }
}

impl ChatActionList {
    pub(crate) fn handle_update(&self, update: types::UpdateChatAction) {
        let imp = self.imp();

        let sender_id = match &update.sender_id {
            enums::MessageSender::User(user) => user.user_id,
            enums::MessageSender::Chat(chat) => chat.chat_id,
        };

        if let Some((position, ..)) = imp.list.borrow_mut().shift_remove_full(&sender_id) {
            self.items_changed(position as u32, 1, 0);
        }

        match update.action {
            enums::ChatAction::Cancel => {}
            action => {
                imp.list.borrow_mut().insert(
                    sender_id,
                    ChatAction::new(action, &update.sender_id, &self.chat()),
                );

                self.items_changed(self.n_items() - 1, 0, 1);
            }
        }
    }

    pub(crate) fn chat(&self) -> Chat {
        self.imp().chat.upgrade().unwrap()
    }

    pub(crate) fn last(&self) -> Option<ChatAction> {
        self.imp()
            .list
            .borrow()
            .last()
            .map(|(_, action)| action)
            .cloned()
    }

    pub(crate) fn group(&self, action: &enums::ChatAction) -> Vec<ChatAction> {
        let discriminant = mem::discriminant(action);
        self.imp()
            .list
            .borrow()
            .values()
            .rev()
            .filter(|action| mem::discriminant(&action.type_().0) == discriminant)
            .cloned()
            .collect()
    }
}
