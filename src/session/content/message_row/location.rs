use gettextrs::gettext;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::MessageContent;

use crate::session::components::AvatarMapMarker;
use crate::session::content::message_row::{MessageBase, MessageBaseImpl, MessageIndicators};
use crate::tdlib::Message;
use crate::utils;

use super::base::MessageBaseExt;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use crate::session::content::message_row::map::Map;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-message-location.ui")]
    pub(crate) struct MessageLocation {
        pub(super) message: RefCell<Option<Message>>,
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) expire_source_id: RefCell<Option<glib::SourceId>>,
        #[template_child]
        pub(super) map: TemplateChild<Map>,
        #[template_child]
        pub(super) indicators: TemplateChild<MessageIndicators>,
        #[template_child]
        pub(super) caption_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) last_updated_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) expire_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) progress_bar: TemplateChild<gtk::ProgressBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageLocation {
        const NAME: &'static str = "ContentMessageLocation";
        type Type = super::MessageLocation;
        type ParentType = MessageBase;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_css_name("messagelocation");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageLocation {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "message",
                    "Message",
                    "The message represented by this row",
                    Message::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => self.message.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageLocation {}
    impl MessageBaseImpl for MessageLocation {}
}

glib::wrapper! {
    pub(crate) struct MessageLocation(ObjectSubclass<imp::MessageLocation>)
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessageLocation {
    type Message = Message;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        if let Some(old_message) = imp.message.take() {
            let handler_id = imp.handler_id.take().unwrap();
            old_message.disconnect(handler_id);
        }

        imp.indicators.set_message(message.clone().upcast());

        // Update the message.
        let handler_id =
            message.connect_content_notify(clone!(@weak self as obj => move |message, _| {
                obj.update(message);
            }));
        imp.handler_id.replace(Some(handler_id));
        self.update(&message);

        imp.message.replace(Some(message));
        self.notify("message");
    }
}

impl MessageLocation {
    fn update(&self, message: &Message) {
        if let MessageContent::MessageLocation(message_) = message.content().0 {
            let imp = self.imp();

            if let Some(source_id) = imp.expire_source_id.take() {
                source_id.remove();
            }

            if message_.live_period > 0 {
                imp.map.set_custom_marker(Some(
                    AvatarMapMarker::from(message.sender().as_user().unwrap()).upcast(),
                ));

                let message_date = message.date();
                if let Some(last_update_date) = self.update_time(message_date, message_.live_period)
                {
                    imp.last_updated_label
                        .set_label(&gettext("updated just now"));

                    let source_id = glib::timeout_add_seconds_local(
                        1,
                        clone!(@weak self as obj => @default-return glib::Continue(false), move || {
                            glib::Continue(match obj.update_time(message_date, message_.live_period) {
                                Some(now) => {
                                    let minutes = now.difference(&last_update_date).as_minutes();
                                    obj.imp().last_updated_label.set_label(&if minutes <= 1 {
                                        gettext("updated just now")
                                    } else if minutes < 60 {
                                        gettext!("updated {} minutes ago", minutes)
                                    } else if minutes == 60 {
                                        gettext("updated an hour ago")
                                    } else {
                                        gettext!(
                                            "updated {} hours and {} minutes ago",
                                            minutes / 60,
                                            minutes % 60
                                        )
                                    });
                                    true
                                },
                                None => false
                            })
                        }),
                    );
                    imp.expire_source_id.replace(Some(source_id));

                    imp.caption_box.set_visible(true);
                    self.add_css_class("with-caption");
                } else {
                    imp.caption_box.set_visible(false);
                    self.remove_css_class("with-caption");
                }
            } else {
                imp.caption_box.set_visible(false);
                self.remove_css_class("with-caption");
            }

            imp.map
                .set_marker_position(message_.location.latitude, message_.location.longitude);
            imp.map.center_marker();
        }
    }

    /// Updates the "expires" labels and the progress bar, and returns the current UTC time if the
    /// live location is not expired, yet.
    fn update_time(&self, message_date: i32, live_period: i32) -> Option<glib::DateTime> {
        let now = glib::DateTime::now_utc().unwrap();
        let expires_in = message_date as i64 + live_period as i64 - now.to_unix();

        let imp = self.imp();
        imp.expire_label
            .set_label(&utils::human_friendly_duration(expires_in as i32));
        imp.progress_bar
            .set_fraction(expires_in as f64 / live_period as f64);

        if expires_in > 0 {
            Some(now)
        } else {
            None
        }
    }
}
