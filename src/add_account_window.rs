use grammers_client::SignInError;
use grammers_client::client::chats::AuthorizationError;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use tokio::runtime;
use tokio::sync::mpsc;

use crate::telegram;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/add_account_window.ui")]
    pub struct AddAccountWindow {
        #[template_child]
        pub content_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub phone_number_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub phone_number_next: TemplateChild<gtk::Button>,
        #[template_child]
        pub authorization_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub confirmation_code_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub confirmation_code_next: TemplateChild<gtk::Button>,
        #[template_child]
        pub sign_in_error_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddAccountWindow {
        const NAME: &'static str = "AddAccountWindow";
        type Type = super::AddAccountWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddAccountWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for AddAccountWindow {}
    impl WindowImpl for AddAccountWindow {}
    impl AdwWindowImpl for AddAccountWindow {}
}

glib::wrapper! {
    pub struct AddAccountWindow(ObjectSubclass<imp::AddAccountWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl AddAccountWindow {
    pub fn new() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create AddAccountWindow")
    }

    pub fn setup_signals(&self, tg_sender: &mpsc::Sender<telegram::EventTG>) {
        let self_ = imp::AddAccountWindow::from_instance(self);

        let phone_number_entry = &*self_.phone_number_entry;
        self_.phone_number_next
            .connect_clicked(glib::clone!(@weak phone_number_entry, @strong tg_sender => move |_| {
                let _ = runtime::Builder::new_current_thread()
                    .build()
                    .unwrap()
                    .block_on(
                        tg_sender.send(telegram::EventTG::SendPhoneNumber(
                        phone_number_entry.get_text().to_string())));
            }));

        let confirmation_code_entry = &*self_.confirmation_code_entry;
        self_.confirmation_code_next
            .connect_clicked(glib::clone!(@weak confirmation_code_entry, @strong tg_sender => move |_| {
                let _ = runtime::Builder::new_current_thread()
                    .build()
                    .unwrap()
                    .block_on(
                        tg_sender.send(telegram::EventTG::SendConfirmationCode(
                        confirmation_code_entry.get_text().to_string())));
            }));
    }

    pub fn navigate_forward(&self) {
        let self_ = imp::AddAccountWindow::from_instance(self);
        self_.content_leaflet.navigate(adw::NavigationDirection::Forward);
    }

    pub fn show_authorization_error(&self, error: AuthorizationError) {
        let self_ = imp::AddAccountWindow::from_instance(self);
        self_.authorization_error_label.set_text(&error.to_string());
    }

    pub fn show_sign_in_error(&self, error: SignInError) {
        let self_ = imp::AddAccountWindow::from_instance(self);
        self_.sign_in_error_label.set_text(&error.to_string());
    }
}
