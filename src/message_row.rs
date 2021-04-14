use grammers_client::types::Message;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::telegram;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct MessageRow {
        pub content_box: RefCell<Option<gtk::Box>>,
        pub message_box: RefCell<Option<gtk::Box>>,
        pub message_picture: RefCell<Option<gtk::Picture>>,

        pub sender_label: RefCell<Option<gtk::Label>>,
        pub sender_avatar: RefCell<Option<adw::Avatar>>,

        pub outgoing: RefCell<Option<bool>>,
        pub sender_id: RefCell<Option<i32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "MessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk::ListBoxRow;
    }

    impl ObjectImpl for MessageRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let content_box = gtk::BoxBuilder::new()
                .spacing(6)
                .build();
            content_box.set_parent(obj);

            let message_box = gtk::BoxBuilder::new()
                .css_classes(vec!("message-box".to_string()))
                .orientation(gtk::Orientation::Vertical)
                .spacing(3)
                .build();
            content_box.append(&message_box);

            *self.content_box.borrow_mut() = Some(content_box);
            *self.message_box.borrow_mut() = Some(message_box);
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
    pub fn new(message: &Message, previous_row: Option<&gtk::ListBoxRow>, gtk_sender: &mpsc::Sender<telegram::GtkEvent>) -> Self {
        let message_row = glib::Object::new(&[("selectable", &false)])
            .expect("Failed to create MessageRow");

        let self_ = imp::MessageRow::from_instance(&message_row);
        let content_box = self_.content_box.borrow().as_ref().unwrap().clone();
        let message_box = self_.message_box.borrow().as_ref().unwrap().clone();

        // Save some parameters for later use
        self_.outgoing.replace(Some(message.outgoing()));
        if let Some(sender) = message.sender() {
            self_.sender_id.replace(Some(sender.id()));
        }

        // Check if the sender widgets needs to be added
        let show_sender = message_row.check_show_sender(previous_row);
        if show_sender {
            // If the sender object exists, use it´s name for the sender,
            // otherwise use the chat name
            let sender_name;
            if let Some(sender) = message.sender() {
                sender_name = sender.name().to_string();
            } else {
                sender_name = message.chat().name().to_string();
            }

            // Add sender label
            let sender_label = gtk::LabelBuilder::new()
                .css_classes(vec!["sender-label".to_string()])
                .ellipsize(pango::EllipsizeMode::End)
                .label(&sender_name)
                .single_line_mode(true)
                .xalign(0.0)
                .build();
            message_box.append(&sender_label);

            // Add sender avatar
            let sender_avatar = adw::AvatarBuilder::new()
                .valign(gtk::Align::Start)
                .show_initials(true)
                .size(36)
                .build();
            sender_label.bind_property("label", &sender_avatar, "text")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            content_box.prepend(&sender_avatar);

            // Save widgets for later use
            self_.sender_avatar.replace(Some(sender_avatar));
            self_.sender_label.replace(Some(sender_label));
        } else if !message.outgoing() {
            // Add margin to the content box to align the message box
            // with the other message boxes that have the avatar
            content_box.set_margin_start(42);
        }

        // Align content box based on the direction of the message
        if message.outgoing() {
            content_box.set_halign(gtk::Align::End);
            content_box.set_margin_start(120);
        } else {
            content_box.set_halign(gtk::Align::Start);
            content_box.set_margin_end(120);
        }

        // Add picture if there´s one in the message
        if let Some(photo) = message.photo() {
            // Create picture widget
            // TODO: improve the size adaptiveness on window size changes
            let message_picture = gtk::PictureBuilder::new()
                .height_request(400)
                .build();
            message_box.append(&message_picture);

            // Load the photo from filesystem
            let chat_id = message.chat().id();
            let path = glib::get_user_special_dir(glib::UserDirectory::Downloads);
            let path = path.join(format!("Telegrand/{}/{}.jpg", chat_id,
                photo.id()));
            message_picture.set_filename(path.to_str());
            self_.message_picture.replace(Some(message_picture));

            // Request high resolution version of the photo to telegram
            let message_id = message.id();
            telegram::send_gtk_event(gtk_sender,
                telegram::GtkEvent::DownloadMessagePhoto(photo, chat_id, message_id));
        }

        // Add message label if there´s text in the message
        if !message.text().is_empty() {
            let message_label = gtk::LabelBuilder::new()
                .css_classes(vec!["text-label".to_string()])
                .label(message.text())
                .selectable(true)
                .wrap(true)
                .wrap_mode(pango::WrapMode::WordChar)
                .xalign(0.0)
                .build();
            message_box.append(&message_label);
        }

        // Add time label
        let mut time = glib::DateTime::from_unix_utc(message.date().timestamp())
            .unwrap().to_local().unwrap().format("%X").unwrap().to_string();
        time.replace_range(5..8, ""); // Remove seconds
        let time_label = gtk::LabelBuilder::new()
            .css_classes(vec!["time-label".to_string()])
            .label(&time)
            .xalign(1.0)
            .build();
        message_box.append(&time_label);

        message_row
    }

    pub fn remove_sender_widgets(&self) {
        let self_ = imp::MessageRow::from_instance(self);
        let content_box = self_.content_box.borrow().as_ref().unwrap().clone();
        let message_box = self_.message_box.borrow().as_ref().unwrap().clone();

        // Remove sender label
        if let Some(sender_label) = self_.sender_label.borrow().as_ref() {
            message_box.remove(sender_label);
        }

        // Remove sender avatar
        if let Some(sender_avatar) = self_.sender_avatar.borrow().as_ref() {
            content_box.remove(sender_avatar);

            // Add margin to the content box to replace the removed avatar
            content_box.set_margin_start(42);
        }

        // Reset saved widgets
        self_.sender_label.replace(None);
        self_.sender_avatar.replace(None);
    }

    pub fn update_picture(&self, path: PathBuf) {
        let self_ = imp::MessageRow::from_instance(self);
        self_.message_picture.borrow().as_ref().unwrap()
            .set_filename(path.to_str());
    }

    pub fn get_sender_id(&self) -> Option<i32> {
        let self_ = imp::MessageRow::from_instance(self);
        self_.sender_id.borrow().as_ref().copied()
    }

    pub fn check_show_sender(&self, previous_row: Option<&gtk::ListBoxRow>) -> bool {
        let self_ = imp::MessageRow::from_instance(self);
        let outgoing = self_.outgoing.borrow().as_ref().unwrap().clone();
        let sender_id = self.get_sender_id();

        // If the message is outgoing don´t show the sender
        if outgoing {
            return false
        }

        // Check if there was a previous row, otherwise show the sender
        if let Some(previous_row) = previous_row {
            // Check if the previous row was a message row, otherwise show
            // the sender
            if let Some(previous_message_row) = previous_row.downcast_ref::<MessageRow>() {
                // Check if the previous message row had a sender object,
                // otherwise show the sender only if the current message has
                // a sender object
                if let Some(previous_sender_id) = previous_message_row.get_sender_id() {
                    // Check if the current message has a sender object,
                    // otherwise show the sender
                    if let Some(sender_id) = sender_id {
                        // If the current sender if different from the previous
                        // one, show the sender
                        return sender_id != previous_sender_id
                    }
                } else {
                    return sender_id.is_some()
                }
            }
        }

        true
    }
}
