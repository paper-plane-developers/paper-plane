use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::types::FormattedText;

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedFormattedText", nullable)]
pub(crate) struct BoxedFormattedText(pub(crate) FormattedText);

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/components-message-entry.ui")]
    pub(crate) struct MessageEntry {
        pub(super) formatted_text: RefCell<Option<BoxedFormattedText>>,
        #[template_child]
        pub(super) overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub(super) placeholder: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) emoji_button: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) text_view: TemplateChild<gtk::TextView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageEntry {
        const NAME: &'static str = "MessageEntry";
        type Type = super::MessageEntry;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageEntry {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("paste-clipboard", &[], <()>::static_type().into()).build(),
                    Signal::builder(
                        "emoji-button-press",
                        &[gtk::Image::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::new(
                        "formatted-text",
                        "Formatted text",
                        "The formatted text of the entry",
                        BoxedFormattedText::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "placeholder-text",
                        "Placeholder text",
                        "The placeholder text of this entry",
                        None,
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
                "formatted-text" => obj.set_formatted_text(value.get().unwrap()),
                "placeholder-text" => obj.set_placeholder_text(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "formatted-text" => obj.formatted_text().to_value(),
                "placeholder-text" => obj.placeholder_text().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.placeholder
                .connect_label_notify(clone!(@weak obj => move |_| obj.notify("placeholder-text")));

            let press = gtk::GestureClick::new();
            press.connect_pressed(clone!(@weak obj => move |_, _, _, _| {
                obj.emit_by_name::<()>("emoji-button-press", &[&*obj.imp().emoji_button]);
            }));
            self.emoji_button.add_controller(&press);

            self.text_view
                .buffer()
                .connect_changed(clone!(@weak obj => move |_| {
                    obj.text_buffer_changed();
                }));

            self.text_view
                .connect_paste_clipboard(clone!(@weak obj => move |_| {
                    obj.emit_by_name::<()>("paste-clipboard", &[]);
                }));
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.overlay.unparent();
        }
    }

    impl WidgetImpl for MessageEntry {}
}

glib::wrapper! {
    pub(crate) struct MessageEntry(ObjectSubclass<imp::MessageEntry>)
        @extends gtk::Widget;
}

impl MessageEntry {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    fn text_buffer_changed(&self) {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        let text: String = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .into();

        if text.is_empty() {
            imp.formatted_text.replace(None);
            imp.placeholder.set_visible(true);
        } else {
            let formatted_text = FormattedText {
                text,
                entities: vec![],
            };
            imp.formatted_text
                .replace(Some(BoxedFormattedText(formatted_text)));

            imp.placeholder.set_visible(false);
        }

        self.notify("formatted-text");
    }

    /// Insert text inside the message entry at the cursor position,
    /// deleting eventual selected text
    pub(crate) fn insert_at_cursor(&self, text: &str) {
        let buffer = self.imp().text_view.buffer();
        buffer.begin_user_action();
        buffer.delete_selection(true, true);
        buffer.insert_at_cursor(text);
        buffer.end_user_action();
    }

    pub(crate) fn formatted_text(&self) -> Option<BoxedFormattedText> {
        self.imp().formatted_text.borrow().clone()
    }

    pub(crate) fn set_formatted_text(&self, formatted_text: Option<BoxedFormattedText>) {
        if self.formatted_text() == formatted_text {
            return;
        }

        let text = formatted_text.map(|f| f.0.text).unwrap_or_default();
        self.imp().text_view.buffer().set_text(&text);
    }

    pub(crate) fn connect_formatted_text_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("formatted-text"), f)
    }

    pub(crate) fn placeholder_text(&self) -> glib::GString {
        self.imp().placeholder.text()
    }

    pub(crate) fn set_placeholder_text(&self, placeholder_text: &str) {
        self.imp().placeholder.set_text(placeholder_text);
    }

    pub(crate) fn connect_paste_clipboard<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("paste-clipboard", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);

            None
        })
    }

    pub(crate) fn connect_emoji_button_press<F: Fn(&Self, gtk::Image) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("emoji-button-press", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let button = values[1].get::<gtk::Image>().unwrap();
            f(&obj, button);

            None
        })
    }
}

impl Default for MessageEntry {
    fn default() -> Self {
        Self::new()
    }
}
