use grammers_client::InputMessage;
use grammers_client::client::messages::MessageIter;
use grammers_client::types::{Dialog, Message};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use std::sync::{Arc, Mutex};
use tokio::runtime;
use tokio::sync::mpsc;

use crate::message_row::MessageRow;
use crate::telegram;
use crate::window::TelegrandWindow;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/chat_page.ui")]
    pub struct ChatPage {
        #[template_child]
        pub chat_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub messages_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub message_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub send_message_button: TemplateChild<gtk::Button>,
        pub dialog: RefCell<Option<Arc<Dialog>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatPage {
        const NAME: &'static str = "ChatPage";
        type Type = super::ChatPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for ChatPage {}
    impl BoxImpl for ChatPage {}
}

glib::wrapper! {
    pub struct ChatPage(ObjectSubclass<imp::ChatPage>)
        @extends gtk::Widget, gtk::Box;
}

impl ChatPage {
    pub fn new(tg_sender: &mpsc::Sender<telegram::EventTG>, dialog: Arc<Dialog>, message_iter: Arc<Mutex<MessageIter>>) -> Self {
        let chat_page = glib::Object::new(&[])
            .expect("Failed to create ChatPage");

        let self_ = imp::ChatPage::from_instance(&chat_page);
        self_.dialog.replace(Some(dialog));

        let _ = runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(
                tg_sender.send(telegram::EventTG::RequestNextMessages(message_iter.clone())));

        let message_entry = &*self_.message_entry;
        let dialog = self_.dialog.borrow().as_ref().unwrap().clone();
        self_.send_message_button
            .connect_clicked(glib::clone!(@weak message_entry, @strong tg_sender => move |_| {
                let message = InputMessage::text(message_entry.get_text());
                message_entry.set_text("");

                let _ = runtime::Builder::new_current_thread()
                    .build()
                    .unwrap()
                    .block_on(
                        tg_sender.send(telegram::EventTG::SendMessage(
                            dialog.clone(), message)));
            }));

        self_.chat_window
            .connect_edge_reached(glib::clone!(@strong tg_sender => move |_, position| {
                if position == gtk::PositionType::Top {
                    let _ = runtime::Builder::new_current_thread()
                        .build()
                        .unwrap()
                        .block_on(
                            tg_sender.send(telegram::EventTG::RequestNextMessages(message_iter.clone())));
                }
            }));

        chat_page
    }

    pub fn update_chat(&self, window: &TelegrandWindow) {
        let self_ = imp::ChatPage::from_instance(self);
        let send_message_button = &*self_.send_message_button;
        window.set_default_widget(Some(send_message_button));
    }

    pub fn append_message(&self, message: &Message) {
        let message_row = MessageRow::new(message);
        let self_ = imp::ChatPage::from_instance(self);
        self_.messages_list.append(&message_row);
    }

    pub fn prepend_message(&self, message: &Message) {
        let message_row = MessageRow::new(message);
        let self_ = imp::ChatPage::from_instance(self);
        self_.messages_list.prepend(&message_row);
    }
}
