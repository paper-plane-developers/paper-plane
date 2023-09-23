use std::cell::RefCell;

use gettextrs::gettext;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use ui::MessageBaseExt;

use crate::i18n::gettext_f;
use crate::model;
use crate::ui;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/message_row/location.ui")]
    pub(crate) struct MessageLocation {
        pub(super) message: glib::WeakRef<model::Message>,
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub(super) expire_source_id: RefCell<Option<glib::SourceId>>,
        #[template_child]
        pub(super) message_bubble: TemplateChild<ui::MessageBubble>,
        #[template_child]
        pub(super) map: TemplateChild<ui::Map>,
        #[template_child]
        pub(super) caption_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) last_updated_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) expire_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) progress_bar: TemplateChild<ui::CircularProgressBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageLocation {
        const NAME: &'static str = "PaplMessageLocation";
        type Type = super::MessageLocation;
        type ParentType = ui::MessageBase;

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
                vec![glib::ParamSpecObject::builder::<model::Message>("message")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "message" => self.obj().set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => self.message.upgrade().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageLocation {}
    impl ui::MessageBaseImpl for MessageLocation {}
}

glib::wrapper! {
    pub(crate) struct MessageLocation(ObjectSubclass<imp::MessageLocation>)
        @extends gtk::Widget, ui::MessageBase;
}

impl MessageBaseExt for MessageLocation {
    type Message = model::Message;

    fn set_message(&self, message: &Self::Message) {
        let imp = self.imp();

        let old_message = imp.message.upgrade();
        if old_message.as_ref() == Some(message) {
            return;
        }

        if let Some(old_message) = old_message {
            let handler_id = imp.handler_id.take().unwrap();
            old_message.disconnect(handler_id);
        }

        imp.message_bubble.update_from_message(message, true);

        // Update the message.
        let handler_id =
            message.connect_content_notify(clone!(@weak self as obj => move |message| {
                obj.update(message);
            }));
        imp.handler_id.replace(Some(handler_id));
        self.update(message);

        imp.message.set(Some(message));
        self.notify("message");
    }
}

impl MessageLocation {
    fn update(&self, message: &model::Message) {
        if let tdlib::enums::MessageContent::MessageLocation(message_) = message.content().0 {
            let imp = self.imp();

            if let Some(source_id) = imp.expire_source_id.take() {
                source_id.remove();
            }

            if message_.live_period > 0 {
                imp.map.set_custom_marker(Some(
                    ui::AvatarMapMarker::from(message.sender().as_user().unwrap()).upcast(),
                ));

                let message_date = message.date();
                if let Some(last_update_date) = self.update_time(message_date, message_.live_period)
                {
                    imp.last_updated_label
                        .set_label(&gettext("updated just now"));

                    let source_id = glib::timeout_add_seconds_local(
                        1,
                        clone!(@weak self as obj => @default-return glib::ControlFlow::Break, move || {
                            match obj.update_time(message_date, message_.live_period) {
                                Some(now) => {
                                    let minutes = now.difference(&last_update_date).as_minutes();
                                    obj.imp().last_updated_label.set_label(&if minutes <= 1 {
                                        gettext("updated just now")
                                    } else if minutes < 60 {
                                        gettext!("updated {} minutes ago", minutes)
                                    } else if minutes == 60 {
                                        gettext("updated an hour ago")
                                    } else {
                                        gettext_f(
                                            "updated {hours} hours and {minutes} minutes ago",
                                            &[
                                                ("hours", &(minutes / 60).to_string()),
                                                ("minutes", &(minutes % 60).to_string()),
                                            ],
                                        )
                                    });

                                    glib::ControlFlow::Continue
                                }
                                None => glib::ControlFlow::Break,
                            }
                        }),
                    );
                    imp.expire_source_id.replace(Some(source_id));

                    imp.caption_box.set_visible(true);
                    imp.message_bubble.add_css_class("live-location");
                } else {
                    imp.caption_box.set_visible(false);
                    imp.message_bubble.remove_css_class("live-location");
                }
            } else {
                imp.caption_box.set_visible(false);
                imp.message_bubble.remove_css_class("live-location");
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
            .set_percentage(expires_in as f64 / live_period as f64);

        if expires_in > 0 {
            Some(now)
        } else {
            None
        }
    }
}
