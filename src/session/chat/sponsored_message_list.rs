use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use tdgrand::{enums, functions};

use crate::session::{chat::SponsoredMessage, Chat};
use crate::utils::do_async;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct SponsoredMessageList {
        pub list: RefCell<Vec<SponsoredMessage>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SponsoredMessageList {
        const NAME: &'static str = "ChatSponsoredMessageList";
        type Type = super::SponsoredMessageList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SponsoredMessageList {}
    impl ListModelImpl for SponsoredMessageList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SponsoredMessage::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get(position as usize)
                .map(glib::object::Cast::upcast_ref)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct SponsoredMessageList(ObjectSubclass<imp::SponsoredMessageList>)
        @implements gio::ListModel;
}

impl Default for SponsoredMessageList {
    fn default() -> Self {
        Self::new()
    }
}

impl SponsoredMessageList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SponsoredMessageList")
    }

    pub fn fetch(&self, chat: &Chat) {
        let chat_id = chat.id();
        let client_id = chat.session().client_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            functions::GetChatSponsoredMessages::new()
                .chat_id(chat_id)
                .send(client_id),
            clone!(@weak self as obj, @weak chat => move |response| async move {
                match response {
                    Ok(enums::SponsoredMessages::SponsoredMessages(result)) => {
                        let self_ = imp::SponsoredMessageList::from_instance(&obj);
                        let position = self_.list.borrow().len() as u32;
                        let added = result.messages.len() as u32;
                        let mut messages = result
                            .messages
                            .into_iter()
                            .map(|m| {
                                let sponsor_chat = chat
                                    .session()
                                    .chat_list()
                                    .get_chat(m.sponsor_chat_id)
                                    .unwrap();
                                SponsoredMessage::new(m, &sponsor_chat)
                            })
                            .collect();

                        self_.list.borrow_mut().append(&mut messages);
                        obj.items_changed(position, 0, added);
                    }
                    Err(e) => {
                        log::error!("Received an error for GetChatSponsoredMessages: {}", e.code);
                    }
                }
            }),
        );
    }

    pub fn clear(&self) {
        let self_ = imp::SponsoredMessageList::from_instance(self);
        let count = self_.list.borrow().len() as u32;

        if count == 0 {
            return;
        }

        self_.list.borrow_mut().clear();
        self.items_changed(0, count, 0);
    }
}
