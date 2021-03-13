use grammers_client::types::Message;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use glib::subclass;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/message_row.ui")]
    pub struct MessageRow {
        #[template_child]
        pub sender_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub message_label: TemplateChild<gtk::Label>,
    }

    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "MessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk::ListBoxRow;
        type Interfaces = ();
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                sender_label: TemplateChild::default(),
                message_label: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for MessageRow {}
    impl ListBoxRowImpl for MessageRow {}
}

glib::wrapper! {
    pub struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl MessageRow {
    pub fn new(message: Message) -> Self {
        let message_row = glib::Object::new(&[])
            .expect("Failed to create MessageRow");

        let self_ = imp::MessageRow::from_instance(&message_row);
        let sender_label = &*self_.sender_label;
        let message_label = &*self_.message_label;

        let sender_name;
        if let Some(sender) = message.sender() {
            sender_name = sender.name().to_string();
        } else {
            sender_name = message.chat().name().to_string();
        }
        sender_label.set_text(&sender_name);

        message_label.set_text(message.text());

        message_row
    }
}
