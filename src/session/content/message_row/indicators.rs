use gettextrs::gettext;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::MessageSendingState;

use crate::tdlib::{Chat, ChatType, Message, MessageInteractionInfo, SponsoredMessage};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    template MessageIndicators {
        layout-manager: BoxLayout {
            spacing: 3;
        };

        Box reply_count_box {
            spacing: 3;

            Image {
                icon-name: "mail-reply-sender-symbolic";
            }

            Label reply_count_label {}
        }

        Label message_info_label {}
        Image sending_state_icon {}
    }
    "#)]
    pub(crate) struct MessageIndicators {
        pub(super) message: RefCell<Option<glib::Object>>,
        pub(super) message_signal_group: OnceCell<glib::SignalGroup>,
        pub(super) interaction_info_signal_group: OnceCell<glib::SignalGroup>,
        pub(super) chat_signal_group: OnceCell<glib::SignalGroup>,
        #[template_child]
        pub(super) reply_count_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) reply_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) message_info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) sending_state_icon: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageIndicators {
        const NAME: &'static str = "MessageIndicators";
        type Type = super::MessageIndicators;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("messageindicators");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageIndicators {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<glib::Object>("message")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "message" => obj.set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "message" => obj.message().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().create_signal_groups();
        }

        fn dispose(&self) {
            let mut child = self.obj().first_child();
            while let Some(child_) = child {
                child = child_.next_sibling();
                child_.unparent();
            }
        }
    }

    impl WidgetImpl for MessageIndicators {}
}

glib::wrapper! {
    pub(crate) struct MessageIndicators(ObjectSubclass<imp::MessageIndicators>)
        @extends gtk::Widget;
}

impl MessageIndicators {
    fn create_signal_groups(&self) {
        let imp = self.imp();

        let message_signal_group = glib::SignalGroup::new(Message::static_type());
        message_signal_group.connect_notify_local(
            Some("is-edited"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_message_info();
            }),
        );
        imp.message_signal_group.set(message_signal_group).unwrap();

        let interaction_info_signal_group =
            glib::SignalGroup::new(MessageInteractionInfo::static_type());
        interaction_info_signal_group.connect_notify_local(
            Some("reply-count"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_reply_count();
            }),
        );
        imp.interaction_info_signal_group
            .set(interaction_info_signal_group)
            .unwrap();

        let chat_signal_group = glib::SignalGroup::new(Chat::static_type());
        chat_signal_group.connect_notify_local(
            Some("last-read-outbox-message-id"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_sending_state();
            }),
        );
        imp.chat_signal_group.set(chat_signal_group).unwrap();
    }

    pub(crate) fn message(&self) -> glib::Object {
        self.imp().message.borrow().clone().unwrap()
    }

    pub(crate) fn set_message(&self, message: glib::Object) {
        let imp = self.imp();
        let old = imp.message.replace(Some(message));

        if old == *imp.message.borrow() {
            return;
        }

        let maybe_message_ref = imp.message.borrow();
        let maybe_message = maybe_message_ref.and_downcast_ref::<Message>();

        imp.message_signal_group
            .get()
            .unwrap()
            .set_target(maybe_message);
        imp.interaction_info_signal_group
            .get()
            .unwrap()
            .set_target(maybe_message.map(|m| m.interaction_info()));
        imp.chat_signal_group
            .get()
            .unwrap()
            .set_target(maybe_message.map(|m| m.chat()).as_ref());

        self.update_reply_count();
        self.update_sending_state();
        self.update_message_info();

        self.notify("message");
    }

    fn update_reply_count(&self) {
        let imp = self.imp();

        let maybe_message_ref = imp.message.borrow();
        let maybe_message = maybe_message_ref.and_downcast_ref::<Message>();

        let is_channel_message = maybe_message
            .filter(|message| {
                matches!(message.chat().type_(), ChatType::Supergroup(data) if data.is_channel())
            })
            .is_some();

        if is_channel_message {
            imp.reply_count_label.set_label("");
            imp.reply_count_box.set_visible(false);
        } else {
            let reply_count = maybe_message
                .map(Message::interaction_info)
                .map(MessageInteractionInfo::reply_count)
                .unwrap_or(0);

            if reply_count > 0 {
                imp.reply_count_label.set_label(&reply_count.to_string());
                imp.reply_count_box.set_visible(true);
            } else {
                imp.reply_count_label.set_label("");
                imp.reply_count_box.set_visible(false);
            }
        }
    }

    fn update_sending_state(&self) {
        let imp = self.imp();
        let maybe_icon_name = imp
            .message
            .borrow()
            .and_downcast_ref::<Message>()
            .filter(|message| message.is_outgoing())
            .map(|message| match message.sending_state() {
                Some(state) => match state.0 {
                    MessageSendingState::Failed(_) => "message-failed-symbolic",
                    MessageSendingState::Pending(_) => "message-pending-symbolic",
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
            });

        if let Some(icon_name) = maybe_icon_name {
            imp.sending_state_icon.set_icon_name(Some(icon_name));
            imp.sending_state_icon.set_visible(true);
        } else {
            imp.sending_state_icon.set_icon_name(None);
            imp.sending_state_icon.set_visible(false);
        }
    }

    fn update_message_info(&self) {
        let imp = self.imp();
        let message = imp.message.borrow();

        let label = if let Some(message) = message.and_downcast_ref::<Message>() {
            let datetime = glib::DateTime::from_unix_utc(message.date() as i64)
                .and_then(|t| t.to_local())
                // Translators: This is a time representation, without seconds.
                // Here you may want to change to a 24-hours representation, based on your locale.
                // You can use this site to learn more: https://www.strfti.me/
                .and_then(|t| t.format(&gettext("%l:%M %p")))
                .unwrap();

            if message.is_edited() {
                format!("{} {}", gettext("edited"), datetime)
            } else {
                datetime.into()
            }
        } else if message.and_downcast_ref::<SponsoredMessage>().is_some() {
            gettext("sponsored")
        } else {
            unreachable!()
        };

        imp.message_info_label.set_label(&label);
    }
}
