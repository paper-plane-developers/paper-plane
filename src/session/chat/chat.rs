use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdgrand::enums::{self, ChatType, Update};
use tdgrand::types::Chat as TelegramChat;
use tdgrand::types::{ChatPhotoInfo, File};

use crate::session::chat::{History, Message};
use crate::Session;

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "BoxedChatType")]
pub struct BoxedChatType(ChatType);

#[derive(Clone, Debug, glib::GBoxed)]
#[gboxed(type_name = "BoxedChatPhoto")]
pub struct BoxedChatPhoto(Option<ChatPhotoInfo>);

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub struct Chat {
        pub id: Cell<i64>,
        pub r#type: OnceCell<ChatType>,
        pub title: RefCell<String>,
        pub photo: RefCell<Option<ChatPhotoInfo>>,
        pub last_message: RefCell<Option<Message>>,
        pub order: Cell<i64>,
        pub unread_count: Cell<i32>,
        pub draft_message: RefCell<String>,
        pub history: OnceCell<History>,
        pub session: OnceCell<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Chat {
        const NAME: &'static str = "Chat";
        type Type = super::Chat;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Chat {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("small-photo-updated", &[], <()>::static_type().into()).build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int64(
                        "id",
                        "Id",
                        "The id of this chat",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "type",
                        "Type",
                        "The type of this chat",
                        BoxedChatType::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "The title of this chat",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpec::new_boxed(
                        "photo",
                        "Photo",
                        "The photo of this chat",
                        BoxedChatPhoto::static_type(),
                        glib::ParamFlags::WRITABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "last-message",
                        "Last Message",
                        "The last message sent on this chat",
                        Message::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_int64(
                        "order",
                        "Order",
                        "The parameter to determine the order of this chat in the chat list",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_int(
                        "unread-count",
                        "Unread Count",
                        "The unread messages count of this chat",
                        std::i32::MIN,
                        std::i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "draft-message",
                        "Draft Message",
                        "The draft message of this chat",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "history",
                        "History",
                        "The message history of this chat",
                        History::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "The session",
                        Session::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "id" => {
                    let id = value.get().unwrap();
                    self.id.set(id);
                }
                "type" => {
                    let r#type = value.get::<BoxedChatType>().unwrap();
                    self.r#type.set(r#type.0).unwrap();
                }
                "title" => {
                    let title = value.get().unwrap();
                    self.title.replace(title);
                }
                "photo" => {
                    let photo = value.get::<BoxedChatPhoto>().unwrap();
                    obj.set_photo(photo.0);
                }
                "last-message" => {
                    let last_message = value.get().unwrap();
                    self.last_message.replace(last_message);
                }
                "order" => {
                    let order = value.get().unwrap();
                    self.order.set(order);
                }
                "unread-count" => {
                    let unread_count = value.get().unwrap();
                    self.unread_count.set(unread_count);
                }
                "draft-message" => {
                    let draft_message = value.get().unwrap();
                    self.draft_message.replace(draft_message);
                }
                "session" => {
                    let session = value.get().unwrap();
                    self.session.set(session).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "title" => self.title.borrow().to_value(),
                "last-message" => self.last_message.borrow().to_value(),
                "order" => self.order.get().to_value(),
                "unread-count" => self.unread_count.get().to_value(),
                "draft-message" => self.draft_message.borrow().to_value(),
                "history" => self.history.get().to_value(),
                "session" => self.session.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.history.set(History::new(obj)).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct Chat(ObjectSubclass<imp::Chat>);
}

impl Chat {
    pub fn new(chat: TelegramChat, session: Session) -> Self {
        let r#type = BoxedChatType(chat.r#type);
        let photo = BoxedChatPhoto(chat.photo);
        glib::Object::new(&[
            ("id", &chat.id),
            ("type", &r#type),
            ("title", &chat.title),
            ("photo", &photo),
            ("session", &session),
        ])
        .expect("Failed to create Chat")
    }

    pub fn handle_update(&self, update: Update) {
        match update {
            Update::NewMessage(_) | Update::MessageContent(_) => {
                self.history().handle_update(update);
            }
            Update::ChatTitle(update) => {
                self.set_title(update.title);
            }
            Update::ChatPhoto(update) => {
                self.set_photo(update.photo);
            }
            Update::ChatLastMessage(update) => {
                match update.last_message {
                    Some(last_message) => {
                        let message = match self.history().message_by_id(last_message.id) {
                            Some(message) => message,
                            None => {
                                let last_message_id = last_message.id;

                                self.history().append(last_message);
                                self.history().message_by_id(last_message_id).unwrap()
                            }
                        };

                        self.set_last_message(Some(message));
                    }
                    None => self.set_last_message(None),
                }

                for position in update.positions {
                    if let enums::ChatList::Main = position.list {
                        self.set_order(position.order);
                        break;
                    }
                }
            }
            Update::ChatPosition(update) => {
                if let enums::ChatList::Main = update.position.list {
                    self.set_order(update.position.order);
                }
            }
            Update::ChatReadInbox(update) => {
                self.set_unread_count(update.unread_count);
            }
            Update::ChatDraftMessage(update) => {
                let mut draft_message = String::new();
                if let Some(message) = update.draft_message {
                    let content = message.input_message_text;
                    if let enums::InputMessageContent::InputMessageText(content) = content {
                        draft_message = content.text.text;
                    }
                }
                self.set_draft_message(draft_message);
            }
            _ => {}
        }
    }

    fn download_small_photo(&self) {
        if let Some(photo) = self.photo() {
            if photo.small.local.can_be_downloaded && !photo.small.local.is_downloading_completed {
                let (sender, receiver) =
                    glib::MainContext::sync_channel::<File>(Default::default(), 5);
                receiver.attach(
                    None,
                    clone!(@weak self as obj => @default-return glib::Continue(false), move |file| {
                        let self_ = imp::Chat::from_instance(&obj);
                        let mut photo = self_.photo.take().unwrap();
                        photo.small = file;

                        self_.photo.replace(Some(photo));
                        obj.emit_by_name("small-photo-updated", &[]).unwrap();

                        glib::Continue(true)
                    }),
                );

                self.session().download_file(photo.small.id, sender);
            }
        }
    }

    pub fn id(&self) -> i64 {
        self.property("id").unwrap().get().unwrap()
    }

    pub fn r#type(&self) -> ChatType {
        let self_ = imp::Chat::from_instance(self);
        self_.r#type.get().unwrap().clone()
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }

    pub fn photo(&self) -> Option<ChatPhotoInfo> {
        let self_ = imp::Chat::from_instance(self);
        self_.photo.borrow().clone()
    }

    pub fn set_photo(&self, photo: Option<ChatPhotoInfo>) {
        let self_ = imp::Chat::from_instance(self);

        self_.photo.replace(photo);
        self.notify("photo");

        self.emit_by_name("small-photo-updated", &[]).unwrap();

        self.download_small_photo();
    }

    fn set_title(&self, title: String) {
        if self.title() != title {
            self.set_property("title", &title).unwrap();
        }
    }

    pub fn last_message(&self) -> Option<Message> {
        self.property("last-message").unwrap().get().unwrap()
    }

    fn set_last_message(&self, last_message: Option<Message>) {
        if self.last_message() != last_message {
            self.set_property("last-message", &last_message).unwrap();
        }
    }

    pub fn order(&self) -> i64 {
        self.property("order").unwrap().get().unwrap()
    }

    fn set_order(&self, order: i64) {
        if self.order() != order {
            self.set_property("order", &order).unwrap();
        }
    }

    pub fn connect_order_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("order"), f)
    }

    pub fn unread_count(&self) -> i32 {
        self.property("unread-count").unwrap().get().unwrap()
    }

    fn set_unread_count(&self, unread_count: i32) {
        if self.unread_count() != unread_count {
            self.set_property("unread-count", &unread_count).unwrap();
        }
    }

    pub fn draft_message(&self) -> String {
        self.property("draft-message").unwrap().get().unwrap()
    }

    fn set_draft_message(&self, draft_message: String) {
        if self.draft_message() != draft_message {
            self.set_property("draft-message", &draft_message).unwrap();
        }
    }

    pub fn history(&self) -> History {
        self.property("history").unwrap().get().unwrap()
    }

    pub fn session(&self) -> Session {
        self.property("session").unwrap().get().unwrap()
    }

    pub fn connect_small_photo_updated<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("small-photo-updated", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);

            None
        })
        .unwrap()
    }
}
