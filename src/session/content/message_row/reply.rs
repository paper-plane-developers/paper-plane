use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::strings;
use crate::tdlib::{Chat, ChatType, Message, MessageSender};
use crate::utils::spawn;

mod imp {
    use gtk::glib::{ParamSpec, Properties, Value};

    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::MessageReply)]
    #[template(string = r#"
    template MessageReply : Widget {
        Separator separator {
            width-request: 2;
        }

        Box labels_box {
            orientation: vertical;

            Label sender_label {
                ellipsize: end;
                xalign: 0;

                styles ["caption-heading"]
            }

            Label message_label {
                ellipsize: end;
                xalign: 0;
                single-line-mode: true;

                styles [
                    "message",
                    "small-body",
                ]
            }
        }
    }
    "#)]
    pub(crate) struct MessageReply {
        pub(super) sender_color_class: RefCell<Option<String>>,
        pub(super) bindings: RefCell<Vec<gtk::ExpressionWatch>>,
        pub(super) is_loading: Cell<bool>,

        #[property(get, set, construct_only)]
        pub(super) chat: RefCell<Option<Chat>>,
        #[property(get, set, construct_only)]
        pub(super) reply_id: Cell<i64>,
        #[property(get, set, construct_only)]
        pub(super) is_outgoing: Cell<bool>,

        #[template_child]
        pub(super) separator: TemplateChild<gtk::Separator>,
        #[template_child]
        pub(super) labels_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) sender_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) message_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageReply {
        const NAME: &'static str = "MessageReply";
        type Type = super::MessageReply;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_css_name("messagereply");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageReply {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.is_loading.set(true);
            self.message_label
                .set_label(&gettextrs::gettext("Loading ..."));
            self.obj().load_message();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for MessageReply {}
}

glib::wrapper! {
    pub(crate) struct MessageReply(ObjectSubclass<imp::MessageReply>)
        @extends gtk::Widget;
}

impl MessageReply {
    pub(crate) fn new(chat: Chat, reply_id: i64, is_outgoing: bool) -> Self {
        glib::Object::builder()
            .property("chat", chat)
            .property("reply-id", reply_id)
            .property("is-outgoing", is_outgoing)
            .build()
    }

    fn load_message(&self) {
        let imp = self.imp();

        let reply_id = imp.reply_id.get();
        let chat = self.chat().unwrap();

        if let Some(message) = chat.message(reply_id) {
            self.update_from_message(&message);
        } else {
            spawn(clone!(@weak self as obj => async move {
                let chat = obj.imp().chat.borrow().clone().unwrap();
                if let Ok(message) = chat.fetch_message(reply_id).await {
                    obj.update_from_message(&message);
                } else {
                    // Message doesn't exist, so we should remove "Loading..." caption
                    // TODO: Impelent it properly using signals
                    obj.imp().message_label.set_label("Deleted message");
                }
            }));
        }

        imp.is_loading.set(false);
    }

    pub(crate) fn set_max_char_width(&self, n_chars: i32) {
        self.imp().message_label.set_max_width_chars(n_chars);
        self.imp().sender_label.set_max_width_chars(n_chars);
    }

    fn update_from_message(&self, message: &Message) {
        let imp = self.imp();
        let mut bindings = imp.bindings.borrow_mut();
        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        // Remove the previous color css class
        let mut sender_color_class = imp.sender_color_class.borrow_mut();
        if let Some(class) = sender_color_class.as_ref() {
            self.remove_css_class(class);
        }
        // Show sender label, if needed
        let show_sender = !matches!(
            message.chat().type_(),
            ChatType::Supergroup(data) if data.is_channel()
        );
        if show_sender {
            let sender_name_expression = message.sender_name_expression();
            let sender_binding =
                sender_name_expression.bind(&*imp.sender_label, "label", glib::Object::NONE);

            bindings.push(sender_binding);

            if !imp.is_outgoing.get() {
                // Color sender label
                if let MessageSender::User(user) = message.sender() {
                    let classes = vec![
                        "sender-text-red",
                        "sender-text-orange",
                        "sender-text-violet",
                        "sender-text-green",
                        "sender-text-cyan",
                        "sender-text-blue",
                        "sender-text-pink",
                    ];

                    let color_class = classes[user.id() as usize % classes.len()];
                    self.add_css_class(color_class);

                    *sender_color_class = Some(color_class.into());
                }
            }
            imp.sender_label.set_visible(true);
        }

        // Set content label expression

        let caption = strings::message_content(message.clone().as_ref());
        imp.message_label.set_label(&caption);

        self.imp().is_loading.set(false);
    }
}
