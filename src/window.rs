use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tokio::sync::mpsc;

use crate::add_account_window::AddAccountWindow;
use crate::chat_page::ChatPage;
use crate::dialog_row::DialogRow;
use crate::telegram;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/window.ui")]
    pub struct TelegrandWindow {
        #[template_child]
        pub chat_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub content_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub back_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub dialog_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub chat_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub add_account_window: TemplateChild<AddAccountWindow>,
        pub dialogs_map: RefCell<HashMap<i32, DialogRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TelegrandWindow {
        const NAME: &'static str = "TelegrandWindow";
        type Type = super::TelegrandWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TelegrandWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for TelegrandWindow {}
    impl WindowImpl for TelegrandWindow {}
    impl ApplicationWindowImpl for TelegrandWindow {}
    impl AdwApplicationWindowImpl for TelegrandWindow {}
}

glib::wrapper! {
    pub struct TelegrandWindow(ObjectSubclass<imp::TelegrandWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl TelegrandWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P, tg_receiver: glib::Receiver<telegram::TelegramEvent>, gtk_sender: mpsc::Sender<telegram::GtkEvent>) -> Self {
        let window = glib::Object::new(&[("application", app)])
            .expect("Failed to create TelegrandWindow");

        let self_ = imp::TelegrandWindow::from_instance(&window);
        self_.add_account_window.setup_signals(&gtk_sender);

        window.setup_signals(&gtk_sender);
        window.setup_tg_receiver(tg_receiver, gtk_sender);

        window
    }

    fn setup_signals(&self, gtk_sender: &mpsc::Sender<telegram::GtkEvent>) {
        let self_ = imp::TelegrandWindow::from_instance(self);

        // Dialog list signal to show the chat on dialog row activation
        self_.dialog_list.connect_row_activated(glib::clone!(@weak self as window, @strong gtk_sender => move |_, row| {
            let self_ = imp::TelegrandWindow::from_instance(&window);
            let dialog_row = row.downcast_ref::<DialogRow>()
                .expect("Row is of wrong type");
            let dialog = dialog_row.get_dialog();
            let chat_id = dialog.chat.id().to_string();
            let chat_name = dialog.chat.name().to_string();
            let chat_page;

            match self_.chat_stack.get_child_by_name(&chat_id) {
                Some(child) => {
                    // Get the existing chat page
                    chat_page = child.downcast()
                        .expect("Child is of wrong type");
                }
                None => {
                    // Create the chat page and add it to the chat stack
                    let message_iter = dialog_row.get_message_iter();
                    chat_page = ChatPage::new(&gtk_sender, dialog, message_iter);
                    self_.chat_stack.add_titled(&chat_page, Some(&chat_id),
                        &chat_name);
                }
            }

            // Update page to prepare it to show
            chat_page.update_chat(&window);

            // Show chat page
            self_.chat_stack.set_visible_child(&chat_page);

            // Set chat name in the titlebar
            self_.chat_name_label.set_text(&chat_name);

            // Navigate to the next page for mobile navigation
            self_.content_leaflet.navigate(adw::NavigationDirection::Forward);
        }));

        // Back button signal for mobile friendly navigation
        let content_leaflet = &*self_.content_leaflet;
        self_.back_button.connect_clicked(glib::clone!(@weak content_leaflet => move |_| {
            content_leaflet.navigate(adw::NavigationDirection::Back);
        }));
    }

    fn setup_tg_receiver(&self, tg_receiver: glib::Receiver<telegram::TelegramEvent>, gtk_sender: mpsc::Sender<telegram::GtkEvent>) {
        tg_receiver.attach(None, glib::clone!(@weak self as window => move |event| {
            let self_ = imp::TelegrandWindow::from_instance(&window);

            match event {
                telegram::TelegramEvent::AccountAuthorized => {
                    self_.add_account_window.hide();

                    telegram::send_gtk_event(&gtk_sender,
                        telegram::GtkEvent::RequestDialogs);
                }
                telegram::TelegramEvent::AccountNotAuthorized => {
                    self_.add_account_window.show();
                }
                telegram::TelegramEvent::NeedConfirmationCode => {
                    self_.add_account_window.navigate_forward();
                }
                telegram::TelegramEvent::PhoneNumberError(error) => {
                    self_.add_account_window.show_phone_number_error(error);
                }
                telegram::TelegramEvent::ConfirmationCodeError(error) => {
                    self_.add_account_window.show_confirmation_code_error(error);
                }
                telegram::TelegramEvent::RequestedDialog(dialog, message_iter) => {
                    // Create dialog row and add it to the dialog list
                    let chat_id = dialog.chat().id();
                    let dialog_row = DialogRow::new(dialog, message_iter);
                    self_.dialog_list.append(&dialog_row);

                    // Insert dialog row to the dialog map to allow getting
                    // the dialog by the chat id when we need it
                    let mut dialogs_map = self_.dialogs_map.borrow_mut();
                    dialogs_map.insert(chat_id, dialog_row);
                }
                telegram::TelegramEvent::RequestedNextMessages(messages, chat_id) => {
                    // Prepend messages to the relative chat page (if it exists)
                    if let Some(child) = self_.chat_stack.get_child_by_name(&chat_id.to_string()) {
                        let chat_page: ChatPage = child.downcast().unwrap();
                        chat_page.prepend_messages(messages);
                    }
                }
                telegram::TelegramEvent::NewMessage(message) => {
                    // Add message to the relative chat page (if it exists)
                    let chat = message.chat();
                    let chat_id = chat.id();
                    if let Some(child) = self_.chat_stack.get_child_by_name(&chat_id.to_string()) {
                        let chat_page: ChatPage = child.downcast().unwrap();
                        chat_page.append_message(&message);
                    }

                    // Update dialog's last message label
                    let dialogs_map = self_.dialogs_map.borrow();
                    let dialog_row = dialogs_map.get(&chat_id).unwrap();
                    let message_text = message.text();
                    dialog_row.set_last_message_text(message_text);

                    if !message.outgoing() {
                        // Send notification about the new incoming message
                        let chat_name = chat.name();
                        let notification = gio::Notification::new("Telegrand");
                        notification.set_title(chat_name);
                        notification.set_body(Some(message_text));
                        let app = window.get_application().unwrap();
                        app.send_notification(Some("new-message"), &notification);

                        // Increment dialog's unread count
                        dialog_row.increment_unread_count();
                    }
                }
            }

            glib::Continue(true)
        }));
    }
}
