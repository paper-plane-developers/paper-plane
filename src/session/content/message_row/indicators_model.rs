use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::tdlib::{Message, SponsoredMessage};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub(crate) struct MessageIndicatorsModel {
        pub(super) message: RefCell<Option<glib::Object>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageIndicatorsModel {
        const NAME: &'static str = "MessageIndicatorsModel";
        type Type = super::MessageIndicatorsModel;
    }

    impl ObjectImpl for MessageIndicatorsModel {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "message",
                        "Message",
                        "The message of the model",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "message-info",
                        "Message info",
                        "The message info of the model",
                        None,
                        glib::ParamFlags::READABLE,
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
                "message" => obj.set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => obj.message().to_value(),
                "message-info" => obj.message_info().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageIndicatorsModel(ObjectSubclass<imp::MessageIndicatorsModel>);
}

impl Default for MessageIndicatorsModel {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageIndicatorsModel {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MessageIndicatorsModel")
    }

    pub(crate) fn message(&self) -> glib::Object {
        self.imp().message.borrow().clone().unwrap()
    }

    pub(crate) fn set_message(&self, message: glib::Object) {
        let imp = self.imp();
        let old = imp.message.replace(Some(message));
        if old != *imp.message.borrow() {
            self.notify("message");
            self.notify("message-info");
        }
    }

    pub(crate) fn message_info(&self) -> String {
        if let Some(message) = self.imp().message.borrow().as_ref() {
            if let Some(message) = message.downcast_ref::<Message>() {
                let datetime = glib::DateTime::from_unix_utc(message.date() as i64)
                    .and_then(|t| t.to_local())
                    .unwrap();

                // Translators: This is a time format for the message timestamp without seconds
                return datetime.format(&gettext("%l:%M %p")).unwrap().into();
            } else if message.downcast_ref::<SponsoredMessage>().is_some() {
                return gettext("sponsored");
            }
        }

        String::new()
    }
}
