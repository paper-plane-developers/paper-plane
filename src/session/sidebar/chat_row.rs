use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

use crate::session::Chat;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-chat-row.ui")]
    pub struct ChatRow {
        pub chat: RefCell<Option<Chat>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatRow {
        const NAME: &'static str = "SidebarChatRow";
        type Type = super::ChatRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "chat",
                        "Chat",
                        "The chat represented by this row",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE,
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
                "chat" => {
                    let chat = value.get().unwrap();
                    obj.set_chat(chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ChatRow {}
    impl BinImpl for ChatRow {}
}

glib::wrapper! {
    pub struct ChatRow(ObjectSubclass<imp::ChatRow>)
        @extends gtk::Widget, adw::Bin;
}

impl ChatRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChatRow")
    }

    pub fn chat(&self) -> Option<Chat> {
        let priv_ = imp::ChatRow::from_instance(self);
        priv_.chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let priv_ = imp::ChatRow::from_instance(self);
        priv_.chat.replace(chat);

        self.notify("chat");
    }
}
