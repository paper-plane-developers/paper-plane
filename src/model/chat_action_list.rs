use std::cell::RefCell;
use std::mem;

use glib::Properties;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use indexmap::IndexMap;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::ChatActionList)]
    pub(crate) struct ChatActionList {
        pub(super) list: RefCell<IndexMap<i64, model::ChatAction>>,
        #[property(get, set, construct_only)]
        pub(super) chat: glib::WeakRef<model::Chat>,
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
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }

    impl ListModelImpl for ChatActionList {
        fn item_type(&self) -> glib::Type {
            model::ChatAction::static_type()
        }

        fn n_items(&self) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
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

impl From<&model::Chat> for ChatActionList {
    fn from(chat: &model::Chat) -> Self {
        glib::Object::builder().property("chat", chat).build()
    }
}

impl ChatActionList {
    pub(crate) fn chat_(&self) -> model::Chat {
        self.chat().unwrap()
    }

    pub(crate) fn last(&self) -> Option<model::ChatAction> {
        self.imp()
            .list
            .borrow()
            .last()
            .map(|(_, action)| action)
            .cloned()
    }

    pub(crate) fn group(&self, action: &tdlib::enums::ChatAction) -> Vec<model::ChatAction> {
        let discriminant = mem::discriminant(action);
        self.imp()
            .list
            .borrow()
            .values()
            .rev()
            .filter(|action| mem::discriminant(&action.action_type().0) == discriminant)
            .cloned()
            .collect()
    }

    pub(crate) fn handle_update(&self, update: tdlib::types::UpdateChatAction) {
        use tdlib::enums::MessageSender::*;

        let imp = self.imp();

        let sender_id = match &update.sender_id {
            User(user) => user.user_id,
            Chat(chat) => chat.chat_id,
        };

        if let Some((position, ..)) = imp.list.borrow_mut().shift_remove_full(&sender_id) {
            self.items_changed(position as u32, 1, 0);
        }

        match update.action {
            tdlib::enums::ChatAction::Cancel => {}
            action => {
                imp.list.borrow_mut().insert(
                    sender_id,
                    model::ChatAction::new(action, &update.sender_id, &self.chat_()),
                );

                self.items_changed(self.n_items() - 1, 0, 1);
            }
        }
    }
}
