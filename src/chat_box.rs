use grammers_client::InputMessage;
use grammers_client::types::Dialog;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use std::sync::Arc;
use tokio::runtime;
use tokio::sync::mpsc;

use crate::telegram;

mod imp {
    use super::*;
    use glib::subclass;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/chat_box.ui")]
    pub struct ChatBox {
        #[template_child]
        pub messages_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub message_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub send_message_button: TemplateChild<gtk::Button>,
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
                message_entry: TemplateChild::default(),
                send_message_button: TemplateChild::default(),
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
    pub fn new(tg_sender: &mpsc::Sender<telegram::EventTG>, dialog: Dialog) -> Self {
        let chat_box = glib::Object::new(&[])
            .expect("Failed to create ChatBox");

        let dialog = Arc::new(dialog);

        let self_ = imp::ChatBox::from_instance(&chat_box);
        let message_entry = &*self_.message_entry;
        let tg_sender_clone = tg_sender.clone();
        self_.send_message_button
            .connect_clicked(glib::clone!(@weak message_entry => move |_| {
                let dialog_clone = dialog.clone();
                let message = InputMessage::text(message_entry.get_text());
                message_entry.set_text("");

                let _ = runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(
                        tg_sender_clone.send(telegram::EventTG::SendMessage(
                        dialog_clone, message)));
            }));

        chat_box
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
