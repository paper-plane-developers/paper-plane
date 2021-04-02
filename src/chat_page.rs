use grammers_client::InputMessage;
use grammers_client::client::messages::MessageIter;
use grammers_client::types::{Dialog, Message};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

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
        pub last_message_id: RefCell<Option<i32>>,
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
                let message = InputMessage::text(message_entry.get_text());
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
        let mut messages_map = self_.messages_map.borrow_mut();

        // Check if the last current message is from the same sender as this
        // message. If it is we don´t want to show the sender widgets.
        let mut show_sender = true;
        if let Some(sender) = message.sender() {
            if let Some(last_message_id) = self_.last_message_id.borrow().as_ref() {
                let last_message_row = messages_map.get(last_message_id).unwrap();
                if let Some(last_sender_id) = last_message_row.get_sender_id() {
                    show_sender = !(sender.id() == last_sender_id);
                }
            }
        }

        // Create the message row and append it to the list
        let message_row = MessageRow::new(message, show_sender, gtk_sender);
        self_.message_list.append(&message_row);

        // Add the message row to the messages map
        let message_id = message.id();
        messages_map.insert(message_id, message_row);

        // This is the last message for now, so save it´s id
        self_.last_message_id.replace(Some(message_id));
    }

    pub fn prepend_messages(&self, messages: Vec<Message>, gtk_sender: &mpsc::Sender<telegram::GtkEvent>) {
        let self_ = imp::ChatPage::from_instance(self);
        let mut message_iter = messages.iter();
        let mut message = message_iter.next();

        // Check if the current oldest message is from the same sender as the
        // first message to prepend. If it is, we need to remove the sender
        // widgets from the former.
        if message.is_some() && message.unwrap().sender().is_some() {
            if let Some(row) = self_.message_list.get_row_at_index(0) {
                let message_row = row.downcast_ref::<MessageRow>()
                    .expect("Row is of wrong type");
                let sender_id = message_row.get_sender_id().unwrap();
                let older_sender_id = message.unwrap().sender().unwrap().id();
                if older_sender_id == sender_id {
                    message_row.remove_sender_widgets();
                }
            }
        }

        while message.is_some() {
            // We want to hide the sender if the older message is the same
            // of the current message. If the message doesn´t have the sender
            // object, it means that it´s from a channel or similar and we
            // want to always show the sender and use the chat name as the
            // sender in that case (this is done in the MessageRow).
            let older_message = message_iter.next();
            let mut show_sender = true;
            if older_message.is_some() && older_message.unwrap().sender().is_some() {
                let sender_id = message.unwrap().sender().unwrap().id();
                let older_sender_id = older_message.unwrap().sender().unwrap().id();
                show_sender = !(sender_id == older_sender_id);
            }

            // Create the message row and prepend it to the list
            let message_row = MessageRow::new(message.unwrap(), show_sender, gtk_sender);
            self_.message_list.prepend(&message_row);

            // If there weren´t no previous messages it means that this is
            // the last message, so save its id
            let mut messages_map = self_.messages_map.borrow_mut();
            let message_id = message.unwrap().id();
            if messages_map.len() == 0 {
                self_.last_message_id.replace(Some(message_id));
            }

            // Add message row to the messages map
            messages_map.insert(message_id, message_row);

            message = older_message;
        }
    }

    pub fn update_message_photo(&self, path: PathBuf, message_id: i32) {
        let self_ = imp::ChatPage::from_instance(self);
        let messages_map = self_.messages_map.borrow();
        let message_row = messages_map.get(&message_id).unwrap();
        message_row.update_photo(path);
    }
}
