use gtk::glib;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;
use tdlib::enums;

use crate::tdlib::Chat;
use crate::tdlib::MessageSender;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedChatActionType")]
pub(crate) struct BoxedChatActionType(pub(crate) enums::ChatAction);

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ChatAction {
        pub(super) type_: OnceCell<BoxedChatActionType>,
        pub(super) sender: OnceCell<MessageSender>,
        pub(super) chat: WeakRef<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatAction {
        const NAME: &'static str = "ChatAction";
        type Type = super::ChatAction;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ChatAction {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::builder::<BoxedChatActionType>("type")
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<MessageSender>("sender")
                        .read_only()
                        .build(),
                    glib::ParamSpecObject::builder::<Chat>("chat")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "type" => obj.type_().to_value(),
                "sender" => obj.sender().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatAction(ObjectSubclass<imp::ChatAction>);
}

impl ChatAction {
    pub(crate) fn new(
        type_: enums::ChatAction,
        sender: &enums::MessageSender,
        chat: &Chat,
    ) -> Self {
        let chat_action: ChatAction = glib::Object::new();
        let imp = chat_action.imp();

        let type_ = BoxedChatActionType(type_);
        let sender = MessageSender::from_td_object(sender, &chat.session());

        imp.type_.set(type_).unwrap();
        imp.sender.set(sender).unwrap();
        imp.chat.set(Some(chat));

        chat_action
    }

    pub(crate) fn type_(&self) -> &BoxedChatActionType {
        self.imp().type_.get().unwrap()
    }

    pub(crate) fn sender(&self) -> &MessageSender {
        self.imp().sender.get().unwrap()
    }

    pub(crate) fn chat(&self) -> Chat {
        self.imp().chat.upgrade().unwrap()
    }
}
