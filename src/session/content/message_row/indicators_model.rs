use gettextrs::gettext;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::{MessageContent, MessageSendingState};

use crate::tdlib::{Message, SponsoredMessage};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub(crate) struct MessageIndicatorsModel {
        pub(super) message: RefCell<Option<glib::Object>>,
        pub(super) is_edited_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) sending_state_handler_id: RefCell<Option<glib::SignalHandlerId>>,
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
                    glib::ParamSpecString::new(
                        "sending-state-icon-name",
                        "Sending state icon name",
                        "The icon name representing the model's message sending state",
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
                "sending-state-icon-name" => obj.sending_state_icon_name().to_value(),
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
            if let Some(handler_id) = imp.is_edited_handler_id.take() {
                old.as_ref()
                    .unwrap()
                    .downcast_ref::<Message>()
                    .unwrap()
                    .disconnect(handler_id);
            }

            if let Some(handler_id) = imp.sending_state_handler_id.take() {
                old.unwrap()
                    .downcast::<Message>()
                    .unwrap()
                    .chat()
                    .disconnect(handler_id);
            }

            if let Ok(message) = self.message().downcast::<Message>() {
                if !message.is_edited() {
                    let handler_id = message.connect_notify_local(
                        Some("is-edited"),
                        clone!(@weak self as obj => move |message, _| {
                            obj.notify("message-info");
                            if message.is_edited() {
                                message.disconnect(obj.imp().is_edited_handler_id.take().unwrap());
                            }
                        }),
                    );
                    imp.is_edited_handler_id.replace(Some(handler_id));
                }

                if message.is_outgoing()
                    && message.id() > message.chat().last_read_outbox_message_id()
                    && !message.chat().is_own_chat()
                {
                    let message_id = message.id();
                    let handler_id = message.chat().connect_notify_local(
                        Some("last-read-outbox-message-id"),
                        clone!(@weak self as obj => move |chat, _| {
                            obj.notify("sending-state-icon-name");
                            if message_id <= chat.last_read_outbox_message_id() {
                                chat.disconnect(obj.imp().sending_state_handler_id.take().unwrap());
                            }
                        }),
                    );
                    imp.sending_state_handler_id.replace(Some(handler_id));
                }
            }

            self.notify("message");
            self.notify("message-info");
            self.notify("sending-state-icon-name");
        }
    }

    pub(crate) fn message_info(&self) -> String {
        if let Some(message) = self.imp().message.borrow().as_ref() {
            if let Some(message) = message.downcast_ref::<Message>() {
                let datetime = glib::DateTime::from_unix_utc(message.date() as i64)
                    .and_then(|t| t.to_local())
                    .unwrap();

                // Translators: This is a time format for the message timestamp without seconds
                let datetime = datetime.format(&gettext("%l:%M %p")).unwrap().into();
                return if !matches!(message.content().0, MessageContent::MessageLocation(_))
                    && message.is_edited()
                {
                    format!("{} {}", gettext("edited"), datetime)
                } else {
                    datetime
                };
            } else if message.downcast_ref::<SponsoredMessage>().is_some() {
                return gettext("sponsored");
            }
        }

        String::new()
    }

    pub(crate) fn sending_state_icon_name(&self) -> String {
        self.imp()
            .message
            .borrow()
            .as_ref()
            .and_then(|message| message.downcast_ref::<Message>())
            .filter(|message| message.is_outgoing())
            .map(|message| match message.sending_state() {
                Some(state) => match state.0 {
                    MessageSendingState::Failed(_) => "message-failed-symbolic",
                    MessageSendingState::Pending => "message-pending-symbolic",
                },
                None => {
                    if message.chat().is_own_chat()
                        || message.id() <= message.chat().last_read_outbox_message_id()
                    {
                        "message-read-symbolic"
                    } else {
                        "message-unread-left-symbolic"
                    }
                }
            })
            .unwrap_or_default()
            .to_string()
    }
}
