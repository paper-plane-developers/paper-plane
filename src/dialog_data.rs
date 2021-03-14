use gtk::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use gtk::prelude::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct DialogData {
        pub chat_id: RefCell<Option<String>>,
        chat_name: RefCell<Option<String>>,
        last_message: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DialogData {
        const NAME: &'static str = "DialogData";
        type Type = super::DialogData;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DialogData {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::string(
                        "chat-id",
                        "ChatId",
                        "ChatId",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::string(
                        "chat-name",
                        "ChatName",
                        "ChatName",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::string(
                        "last-message",
                        "LastMessage",
                        "LastMessage",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.get_name() {
                "chat-id" => {
                    let chat_id = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.chat_id.replace(chat_id);
                }
                "chat-name" => {
                    let chat_name = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.chat_name.replace(chat_name);
                }
                "last-message" => {
                    let last_message = value
                        .get()
                        .expect("type conformity checked by `Object::set_property`");
                    self.last_message.replace(last_message);
                }
                _ => unimplemented!(),
            }
        }

        fn get_property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.get_name() {
                "chat-id" => self.chat_id.borrow().to_value(),
                "chat-name" => self.chat_name.borrow().to_value(),
                "last-message" => self.last_message.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct DialogData(ObjectSubclass<imp::DialogData>);
}

impl DialogData {
    pub fn new(chat_id: &str, chat_name: &str, last_message: &str) -> Self {
        glib::Object::new(&[("chat-id", &chat_id), ("chat-name", &chat_name), ("last-message", &last_message)])
            .expect("Failed to create DialogData")
    }

    pub fn get_chat_id(&self) -> String {
        let self_ = imp::DialogData::from_instance(self);
        if let Some(chat_id) = &*self_.chat_id.borrow() {
            return chat_id.to_string();
        }
        "".to_string()
    }
}
