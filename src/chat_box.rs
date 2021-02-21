use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use glib::subclass;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/chat_box.ui")]
    pub struct ChatBox {
        #[template_child]
        pub messages_box: TemplateChild<gtk::Box>,
    }

    impl ObjectSubclass for ChatBox {
        const NAME: &'static str = "ChatBox";
        type Type = super::ChatBox;
        type ParentType = gtk::Box;
        type Interfaces = ();
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                messages_box: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatBox {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for ChatBox {}
    impl BoxImpl for ChatBox {}
}

glib::wrapper! {
    pub struct ChatBox(ObjectSubclass<imp::ChatBox>)
        @extends gtk::Widget, gtk::Box;
}

impl ChatBox {
    pub fn new() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create ChatBox")
    }

    pub fn add_message(&self, message_text: &str, outgoing: bool) {
        let message_label = gtk::Label::new(Some(message_text));
        if outgoing {
            message_label.set_halign(gtk::Align::End);
        } else {
            message_label.set_halign(gtk::Align::Start);
        }

        let self_ = imp::ChatBox::from_instance(self);
        self_.messages_box.append(&message_label);
    }
}
