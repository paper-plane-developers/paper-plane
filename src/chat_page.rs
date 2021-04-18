use grammers_client::InputMessage;
use grammers_client::client::messages::MessageIter;
use grammers_client::types::{Dialog, Message};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::message_row::MessageRow;
use crate::telegram;
use crate::window::TelegrandWindow;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/chat_page.ui")]
    pub struct ChatPage {
        #[template_child]
        pub messages_scroll: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub message_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub message_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub send_message_button: TemplateChild<gtk::Button>,

        pub dialog: RefCell<Option<Arc<Dialog>>>,
        pub messages_map: RefCell<HashMap<i32, MessageRow>>,
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
    pub fn new(gtk_sender: &mpsc::Sender<telegram::GtkEvent>, dialog: Arc<Dialog>, message_iter: Arc<Mutex<MessageIter>>) -> Self {
        let chat_page = glib::Object::new(&[])
            .expect("Failed to create ChatPage");

        let self_ = imp::ChatPage::from_instance(&chat_page);
        self_.dialog.replace(Some(dialog));

        let dialog = self_.dialog.borrow().as_ref().unwrap().clone();
        let chat_id = dialog.chat().id();
        telegram::send_gtk_event(gtk_sender,
            telegram::GtkEvent::RequestNextMessages(message_iter.clone(), chat_id));

        let message_entry = &*self_.message_entry;
        self_.send_message_button
            .connect_clicked(glib::clone!(@weak message_entry, @strong gtk_sender => move |_| {
                let message = InputMessage::text(message_entry.text());
                message_entry.set_text("");

                telegram::send_gtk_event(&gtk_sender,
                    telegram::GtkEvent::SendMessage(dialog.clone(), message));
            }));

        self_.messages_scroll
            .connect_edge_reached(glib::clone!(@strong gtk_sender => move |_, position| {
                if position == gtk::PositionType::Top {
                    telegram::send_gtk_event(&gtk_sender,
                        telegram::GtkEvent::RequestNextMessages(message_iter.clone(), chat_id));
                }
            }));

        chat_page
    }

    pub fn update_chat(&self, window: &TelegrandWindow) {
        let self_ = imp::ChatPage::from_instance(self);
        let send_message_button = &*self_.send_message_button;
        window.set_default_widget(Some(send_message_button));
    }

    pub fn append_message(&self, message: &Message, gtk_sender: &mpsc::Sender<telegram::GtkEvent>) {
        let self_ = imp::ChatPage::from_instance(self);

        // Get the previous row, if it exists. Its previous row should
        // be the last child of the message list, so do all the checking to
        // see if it´s actually a row.
        let previous_row;
        let last_child = self_.message_list.last_child();
        if let Some(last_child) = last_child {
            if let Ok(last_child) = last_child.downcast::<gtk::ListBoxRow>() {
                previous_row = Some(last_child);
            } else {
                previous_row = None;
            }
        } else {
            previous_row = None;
        }

        // Create message row and add it to the message list
        let message_row = MessageRow::new(message, previous_row.as_ref(), gtk_sender);
        self_.message_list.append(&message_row);

        // Add message row to the messages map
        let mut messages_map = self_.messages_map.borrow_mut();
        messages_map.insert(message.id(), message_row);
    }

    pub fn prepend_messages(&self, messages: Vec<Message>, gtk_sender: &mpsc::Sender<telegram::GtkEvent>) {
        let self_ = imp::ChatPage::from_instance(self);
        let mut messages_map = self_.messages_map.borrow_mut();
        let past_first_child = self_.message_list.first_child();

        // Reverse insert messages (oldest to newest) to automatically check
        // the previous row for every added message
        let mut previous_row = None;
        for (index, message) in messages.iter().rev().enumerate() {
            // Create message row and add it to the message list
            let message_row = MessageRow::new(message, previous_row, gtk_sender);
            self_.message_list.insert(&message_row, index as i32);

            // Add message row to the messages map
            messages_map.insert(message.id(), message_row);

            // Get back the message row from the map and set it as previous row
            let message_row = messages_map.get(&message.id()).unwrap();
            previous_row = Some(message_row.upcast_ref());
        }

        // Check if the past first child exists and it´s also a message row.
        // In that case check if it´s sender widgets needs to be removed
        // by passing it´s new previous row.
        if let Some(past_first_child) = past_first_child {
            if let Ok(previous_first_message_row) = past_first_child.downcast::<MessageRow>() {
                if !previous_first_message_row.check_show_sender(previous_row) {
                    previous_first_message_row.remove_sender_widgets();
                }
            }
        }
    }

    pub fn update_message_picture(&self, path: PathBuf, message_id: i32) {
        let self_ = imp::ChatPage::from_instance(self);
        let messages_map = self_.messages_map.borrow();
        let message_row = messages_map.get(&message_id).unwrap();
        message_row.update_picture(path);
    }
}
