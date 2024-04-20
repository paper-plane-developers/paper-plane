mod background;
mod chat_action_bar;
mod chat_history;
mod chat_history_row;
mod chat_info_window;
mod event_row;
mod message_row;
mod send_media_window;

use std::sync::OnceLock;

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

pub(crate) use self::background::Background;
pub(crate) use self::chat_action_bar::ChatActionBar;
pub(crate) use self::chat_history::ChatHistory;
pub(crate) use self::chat_history_row::ChatHistoryRow;
pub(crate) use self::chat_info_window::ChatInfoWindow;
pub(crate) use self::event_row::EventRow;
pub(crate) use self::message_row::MediaPicture;
pub(crate) use self::message_row::MessageBase;
pub(crate) use self::message_row::MessageBaseExt;
pub(crate) use self::message_row::MessageBaseImpl;
pub(crate) use self::message_row::MessageBubble;
pub(crate) use self::message_row::MessageDocument;
pub(crate) use self::message_row::MessageDocumentStatusIndicator;
pub(crate) use self::message_row::MessageIndicators;
pub(crate) use self::message_row::MessageLabel;
pub(crate) use self::message_row::MessageLocation;
pub(crate) use self::message_row::MessagePhoto;
pub(crate) use self::message_row::MessageReply;
pub(crate) use self::message_row::MessageSticker;
pub(crate) use self::message_row::MessageText;
pub(crate) use self::message_row::MessageVenue;
pub(crate) use self::message_row::MessageVideo;
pub(crate) use self::message_row::Row as MessageRow;
pub(crate) use self::send_media_window::SendMediaWindow;
use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/mod.ui")]
    pub(crate) struct Content {
        pub(super) chat: glib::WeakRef<model::Chat>,
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) unselected_chat_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) chat_history: TemplateChild<ChatHistory>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Content {
        const NAME: &'static str = "Content";
        type Type = super::Content;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Content {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecObject::builder::<model::Chat>("chat")
                    .explicit_notify()
                    .build()]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "chat" => {
                    let chat = value.get().unwrap();
                    obj.set_chat(chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Content {}
    impl BinImpl for Content {}
}

glib::wrapper! {
    pub(crate) struct Content(ObjectSubclass<imp::Content>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Content {
    fn default() -> Self {
        Self::new()
    }
}

impl Content {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn handle_paste_action(&self) {
        self.imp().chat_history.handle_paste_action();
    }

    pub(crate) fn chat(&self) -> Option<model::Chat> {
        self.imp().chat.upgrade()
    }

    fn set_chat(&self, chat: Option<&model::Chat>) {
        if self.chat().as_ref() == chat {
            return;
        }

        let imp = self.imp();
        if chat.is_some() {
            imp.stack.set_visible_child(&imp.chat_history.get());
        } else {
            imp.stack.set_visible_child(&imp.unselected_chat_view.get());
        }

        imp.chat.set(chat);

        self.notify("chat");
    }
}
