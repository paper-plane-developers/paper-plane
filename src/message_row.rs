use grammers_client::types::Message;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use gtk::pango;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::telegram;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct MessageRow {
        pub message_hbox: RefCell<Option<gtk::Box>>,
        pub message_vbox: RefCell<Option<gtk::Box>>,

        pub message_picture: RefCell<Option<gtk::Picture>>,
        pub time_label: RefCell<Option<gtk::Label>>,

        pub sender_label: RefCell<Option<gtk::Label>>,
        pub sender_avatar: RefCell<Option<adw::Avatar>>,

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

            let message_hbox = gtk::BoxBuilder::new()
                // Margin when there´s no avatar (avatar size + hbox spacing)
                .margin_start(48)
                .spacing(12)
                .build();
            message_hbox.set_parent(obj);

            let message_vbox = gtk::BoxBuilder::new()
                .hexpand(true)
                .orientation(gtk::Orientation::Vertical)
                .build();
            message_hbox.append(&message_vbox);

            let time_label = gtk::LabelBuilder::new()
                .css_classes(vec![String::from("caption")])
                .yalign(0.0)
                .build();
            message_hbox.append(&time_label);

            *self.message_hbox.borrow_mut() = Some(message_hbox);
            *self.message_vbox.borrow_mut() = Some(message_vbox);
            *self.time_label.borrow_mut() = Some(time_label);
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
    pub fn new(message: &Message, show_sender: bool, gtk_sender: &mpsc::Sender<telegram::GtkEvent>) -> Self {
        let message_row = glib::Object::new(&[("selectable", &false)])
            .expect("Failed to create MessageRow");

        let self_ = imp::MessageRow::from_instance(&message_row);

        // Create sender widgets if the need them
        if show_sender {
            let sender_name;
            if let Some(sender) = message.sender() {
                sender_name = sender.name().to_string();
            } else {
                sender_name = message.chat().name().to_string();
            }
            message_row.create_sender_widgets(&sender_name);
        }

        // Add the photo if there´s one in the message
        if let Some(photo) = message.photo() {
            // Load the photo from filesystem
            let path = glib::get_user_special_dir(glib::UserDirectory::Downloads);
            let path = path.join(format!("Telegrand/{}/{}.jpg", message.chat().id(),
                photo.id()));
            // TODO: improve the size adaptiveness on window size changes
            let picture = gtk::PictureBuilder::new()
                .height_request(300)
                .build();
                picture.set_filename(path.to_str());
            self_.message_vbox.borrow().as_ref().unwrap().append(&picture);
            self_.message_picture.replace(Some(picture));

            // Request high resolution version of the photo to telegram
            let chat_id = message.chat().id();
            let message_id = message.id();
            telegram::send_gtk_event(gtk_sender,
                telegram::GtkEvent::DownloadMessagePhoto(photo, chat_id, message_id));
        }

        // Add message label if there´s text in the message
        if message.text().len() > 0 {
            let message_label = gtk::LabelBuilder::new()
                .label(message.text())
                .selectable(true)
                .wrap(true)
                .wrap_mode(pango::WrapMode::WordChar)
                .xalign(0.0)
                .build();
            self_.message_vbox.borrow().as_ref().unwrap().append(&message_label);
        }

        // Set time text
        let time = message.date().format("%H:%M").to_string();
        self_.time_label.borrow().as_ref().unwrap().set_text(&time);

        // Save the sender id
        if let Some(sender) = message.sender() {
            self_.sender_id.replace(Some(sender.id()));
        }

        message_row
    }

    fn create_sender_widgets(&self, sender_name: &str) {
        let self_ = imp::MessageRow::from_instance(self);

        // Create sender label
        let sender_label = gtk::LabelBuilder::new()
            .css_classes(vec![String::from("heading")])
            .ellipsize(pango::EllipsizeMode::End)
            .label(sender_name)
            .single_line_mode(true)
            .xalign(0.0)
            .build();
        self_.message_vbox.borrow().as_ref().unwrap().prepend(&sender_label);

        // Create sender avatar
        let sender_avatar = adw::AvatarBuilder::new()
            .valign(gtk::Align::Start)
            .show_initials(true)
            .size(36)
            .build();
        sender_label.bind_property("label", &sender_avatar, "text")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        self_.message_hbox.borrow().as_ref().unwrap().prepend(&sender_avatar);

        // Save widgets for later use
        self_.sender_label.replace(Some(sender_label));
        self_.sender_avatar.replace(Some(sender_avatar));

        // Remove margin from the message as we now have an avatar
        self_.message_hbox.borrow().as_ref().unwrap().set_margin_start(0);
    }

    pub fn remove_sender_widgets(&self) {
        let self_ = imp::MessageRow::from_instance(self);

        // Remove widgets from the relative boxes
        self_.message_vbox.borrow().as_ref().unwrap().remove(
            self_.sender_label.borrow().as_ref().unwrap());
        self_.message_hbox.borrow().as_ref().unwrap().remove(
            self_.sender_avatar.borrow().as_ref().unwrap());

        // Reset saved widgets
        self_.sender_label.replace(None);
        self_.sender_avatar.replace(None);

        // Add margin to the message as we have removed the avatar
        self_.message_hbox.borrow().as_ref().unwrap().set_margin_start(48);
    }

    pub fn get_sender_id(&self) -> Option<i32> {
        let self_ = imp::MessageRow::from_instance(self);
        self_.sender_id.borrow().as_ref().copied()
    }

    pub fn update_photo(&self, path: PathBuf) {
        let self_ = imp::MessageRow::from_instance(self);
        self_.message_picture.borrow().as_ref().unwrap()
            .set_filename(path.to_str());
    }
}
