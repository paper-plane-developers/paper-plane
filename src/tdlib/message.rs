use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::{MessageSender as TdMessageSender, Update};
use tdlib::functions;
use tdlib::types::{Error as TdError, Message as TdMessage};

use crate::tdlib::{
    BoxedMessageContent, BoxedMessageSendingState, Chat, MessageForwardInfo, MessageForwardOrigin,
    User,
};
use crate::{expressions, Session};

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "MessageSender")]
pub(crate) enum MessageSender {
    User(User),
    Chat(Chat),
}

impl MessageSender {
    pub(crate) fn from_td_object(sender: &TdMessageSender, session: &Session) -> Self {
        match sender {
            TdMessageSender::User(data) => {
                let user = session.user(data.user_id);
                MessageSender::User(user)
            }
            TdMessageSender::Chat(data) => {
                let chat = session.chat(data.chat_id);
                MessageSender::Chat(chat)
            }
        }
    }

    pub(crate) fn as_user(&self) -> Option<&User> {
        match self {
            MessageSender::User(user) => Some(user),
            _ => None,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        match self {
            Self::User(user) => user.id(),
            Self::Chat(chat) => chat.id(),
        }
    }
}

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::{Lazy, OnceCell};
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct Message {
        pub(super) id: Cell<i64>,
        pub(super) sender: OnceCell<MessageSender>,
        pub(super) is_outgoing: Cell<bool>,
        pub(super) can_be_edited: Cell<bool>,
        pub(super) can_be_deleted_only_for_self: Cell<bool>,
        pub(super) can_be_deleted_for_all_users: Cell<bool>,
        pub(super) sending_state: RefCell<Option<BoxedMessageSendingState>>,
        pub(super) date: Cell<i32>,
        pub(super) content: RefCell<Option<BoxedMessageContent>>,
        pub(super) is_edited: Cell<bool>,
        pub(super) chat: WeakRef<Chat>,
        pub(super) forward_info: OnceCell<Option<MessageForwardInfo>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Message {
        const NAME: &'static str = "Message";
        type Type = super::Message;
    }

    impl ObjectImpl for Message {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("id").read_only().build(),
                    glib::ParamSpecBoxed::builder::<MessageSender>("sender")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-outgoing")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("can-be-edited")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("can-be-deleted-only-for-self")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("can-be-deleted-for-all-users")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<BoxedMessageSendingState>("sending-state")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt::builder("date").read_only().build(),
                    glib::ParamSpecBoxed::builder::<BoxedMessageContent>("content")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-edited")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<Chat>("chat")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<MessageForwardInfo>("forward-info")
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
                "sender" => obj.sender().to_value(),
                "is-outgoing" => obj.is_outgoing().to_value(),
                "can-be-edited" => obj.can_be_edited().to_value(),
                "can-be-deleted-only-for-self" => obj.can_be_deleted_only_for_self().to_value(),
                "can-be-deleted-for-all-users" => obj.can_be_deleted_for_all_users().to_value(),
                "sending-state" => obj.sending_state().to_value(),
                "date" => obj.date().to_value(),
                "content" => obj.content().to_value(),
                "is-edited" => obj.is_edited().to_value(),
                "chat" => obj.chat().to_value(),
                "forward-info" => obj.forward_info().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Message(ObjectSubclass<imp::Message>);
}

impl Message {
    pub(crate) fn new(td_message: TdMessage, chat: &Chat) -> Self {
        let message: Message = glib::Object::builder().build();
        let imp = message.imp();

        let sender = MessageSender::from_td_object(&td_message.sender_id, &chat.session());
        let sending_state = td_message.sending_state.map(BoxedMessageSendingState);
        let content = BoxedMessageContent(td_message.content);
        let is_edited = td_message.edit_date > 0;
        let forward_info = td_message
            .forward_info
            .map(|forward_info| MessageForwardInfo::from_td_object(forward_info, chat));

        imp.id.set(td_message.id);
        imp.sender.set(sender).unwrap();
        imp.is_outgoing.set(td_message.is_outgoing);
        imp.can_be_edited.set(td_message.can_be_edited);
        imp.can_be_deleted_only_for_self
            .set(td_message.can_be_deleted_only_for_self);
        imp.can_be_deleted_for_all_users
            .set(td_message.can_be_deleted_for_all_users);
        imp.sending_state.replace(sending_state);
        imp.date.set(td_message.date);
        imp.content.replace(Some(content));
        imp.is_edited.set(is_edited);
        imp.chat.set(Some(chat));
        imp.forward_info.set(forward_info).unwrap();

        message
    }

    pub(crate) fn handle_update(&self, update: Update) {
        match update {
            Update::MessageContent(data) => {
                let new_content = BoxedMessageContent(data.new_content);
                self.set_content(new_content);
            }
            Update::MessageEdited(data) => self.set_is_edited(data.edit_date > 0),
            _ => {}
        }
    }

    pub(crate) async fn delete(&self, revoke: bool) -> Result<(), TdError> {
        functions::delete_messages(
            self.chat().id(),
            vec![self.id()],
            revoke,
            self.chat().session().client_id(),
        )
        .await
    }

    pub(crate) fn id(&self) -> i64 {
        self.imp().id.get()
    }

    pub(crate) fn sender(&self) -> &MessageSender {
        self.imp().sender.get().unwrap()
    }

    pub(crate) fn is_outgoing(&self) -> bool {
        self.imp().is_outgoing.get()
    }

    pub(crate) fn can_be_edited(&self) -> bool {
        self.imp().can_be_edited.get()
    }

    pub(crate) fn can_be_deleted_only_for_self(&self) -> bool {
        self.imp().can_be_deleted_only_for_self.get()
    }

    pub(crate) fn can_be_deleted_for_all_users(&self) -> bool {
        self.imp().can_be_deleted_for_all_users.get()
    }

    pub(crate) fn sending_state(&self) -> Option<BoxedMessageSendingState> {
        self.imp().sending_state.borrow().clone()
    }

    pub(crate) fn date(&self) -> i32 {
        self.imp().date.get()
    }

    pub(crate) fn content(&self) -> BoxedMessageContent {
        self.imp().content.borrow().as_ref().unwrap().to_owned()
    }

    fn set_content(&self, content: BoxedMessageContent) {
        if self.imp().content.borrow().as_ref() == Some(&content) {
            return;
        }
        self.imp().content.replace(Some(content));
        self.notify("content");
    }

    pub(crate) fn is_edited(&self) -> bool {
        self.imp().is_edited.get()
    }

    fn set_is_edited(&self, is_edited: bool) {
        if self.is_edited() == is_edited {
            return;
        }
        self.imp().is_edited.set(is_edited);
        self.notify("is-edited");
    }

    pub(crate) fn connect_content_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("content"), f)
    }

    pub(crate) fn chat(&self) -> Chat {
        self.imp().chat.upgrade().unwrap()
    }

    pub(crate) fn forward_info(&self) -> Option<&MessageForwardInfo> {
        self.imp().forward_info.get().unwrap().as_ref()
    }

    pub(crate) fn sender_name_expression(&self) -> gtk::Expression {
        match self.sender() {
            MessageSender::User(user) => {
                let user_expression = gtk::ConstantExpression::new(user);
                expressions::user_display_name(&user_expression)
            }
            MessageSender::Chat(chat) => gtk::ConstantExpression::new(chat)
                .chain_property::<Chat>("title")
                .upcast(),
        }
    }

    pub(crate) fn sender_display_name_expression(&self) -> gtk::Expression {
        if self.chat().is_own_chat() {
            self.forward_info()
                .map(MessageForwardInfo::origin)
                .map(|forward_origin| match forward_origin {
                    MessageForwardOrigin::User(user) => {
                        let user_expression = gtk::ObjectExpression::new(user);
                        expressions::user_display_name(&user_expression)
                    }
                    MessageForwardOrigin::Chat { chat, .. }
                    | MessageForwardOrigin::Channel { chat, .. } => {
                        gtk::ConstantExpression::new(chat)
                            .chain_property::<Chat>("title")
                            .upcast()
                    }
                    MessageForwardOrigin::HiddenUser { sender_name }
                    | MessageForwardOrigin::MessageImport { sender_name } => {
                        gtk::ConstantExpression::new(&sender_name).upcast()
                    }
                })
                .unwrap_or_else(|| self.sender_display_name_expression())
        } else {
            self.sender_name_expression()
        }
    }
}
