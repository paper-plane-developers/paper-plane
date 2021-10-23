mod chat_history;
mod event_row;
mod item_row;
mod message_bubble;
mod message_indicators;
mod message_label;
mod message_row;
mod message_sticker;
mod send_message_area;
mod user_dialog;

use self::chat_history::ChatHistory;
use self::event_row::EventRow;
use self::item_row::ItemRow;
use self::message_bubble::MessageBubble;
use self::message_indicators::MessageIndicators;
use self::message_label::MessageLabel;
use self::message_row::MessageRow;
use self::message_sticker::MessageSticker;
use self::send_message_area::SendMessageArea;
use self::user_dialog::UserDialog;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::session::Chat;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content.ui")]
    pub struct Content {
        pub compact: Cell<bool>,
        pub chat: RefCell<Option<Chat>>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub unselected_chat: TemplateChild<gtk::Box>,
        #[template_child]
        pub chat_history: TemplateChild<ChatHistory>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Content {
        const NAME: &'static str = "Content";
        type Type = super::Content;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ChatHistory::static_type();
            Self::bind_template(klass);

            klass.install_action("content.go-back", None, move |widget, _, _| {
                widget.set_chat(None);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Content {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boolean(
                        "compact",
                        "Compact",
                        "Wheter a compact view is used or not",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "chat",
                        "Chat",
                        "The chat currently shown",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "compact" => {
                    let compact = value.get().unwrap();
                    self.compact.set(compact);
                }
                "chat" => {
                    let chat = value.get().unwrap();
                    obj.set_chat(chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Content {}
    impl BinImpl for Content {}
}

glib::wrapper! {
    pub struct Content(ObjectSubclass<imp::Content>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Content {
    fn default() -> Self {
        Self::new()
    }
}

impl Content {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Content")
    }

    pub fn chat(&self) -> Option<Chat> {
        let self_ = imp::Content::from_instance(self);
        self_.chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let self_ = imp::Content::from_instance(self);
        if chat.is_some() {
            self_.stack.set_visible_child(&self_.chat_history.get());
        } else {
            self_.stack.set_visible_child(&self_.unselected_chat.get());
        }

        self_.chat.replace(chat);

        self.notify("chat");
    }
}
