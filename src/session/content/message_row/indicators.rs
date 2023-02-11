use super::indicators_model::MessageIndicatorsModel;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use glib::clone;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    template MessageIndicators {
        layout-manager: BoxLayout {
            spacing: 3;
        };

        Label {
            label: bind MessageIndicators.model.message-info;
        }

        Image sending_state_icon {
            icon-name: bind MessageIndicators.model.sending-state-icon-name;
        }
    }
    "#)]
    pub(crate) struct MessageIndicators {
        pub(super) model: MessageIndicatorsModel,
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
                vec![
                    glib::ParamSpecObject::builder::<glib::Object>("message")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<MessageIndicatorsModel>("model")
                        .read_only()
                        .build(),
                ]
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
                "model" => obj.model().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            self.model.connect_notify_local(
                Some("message"),
                clone!(@weak obj => move |_, _| {
                    obj.notify("message");
                }),
            );

            self.sending_state_icon
                .connect_icon_name_notify(|sending_state_icon| {
                    sending_state_icon.set_visible(
                        sending_state_icon
                            .icon_name()
                            .map(|icon_name| !icon_name.is_empty())
                            .unwrap_or(false),
                    )
                });
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
    pub(crate) fn message(&self) -> glib::Object {
        self.imp().model.message()
    }

    pub(crate) fn set_message(&self, message: glib::Object) {
        self.imp().model.set_message(message);
    }

    pub(crate) fn model(&self) -> &MessageIndicatorsModel {
        &self.imp().model
    }
}
