use grammers_client::SignInError;
use grammers_client::client::chats::AuthorizationError;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
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
        pub previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub phone_number_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub phone_number_error_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub confirmation_code_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub confirmation_code_error_label: TemplateChild<gtk::Label>,
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

    pub fn setup_signals(&self, gtk_sender: &mpsc::Sender<telegram::GtkEvent>) {
        let self_ = imp::AddAccountWindow::from_instance(self);

        let content_leaflet = &*self_.content_leaflet;
        let previous_button = &*self_.previous_button;
        self_.previous_button
            .connect_clicked(glib::clone!(@weak content_leaflet, @weak previous_button => move |_| {
                content_leaflet.navigate(adw::NavigationDirection::Back);

                let page_name = content_leaflet.visible_child_name().unwrap();
                if page_name == "phone_number_page" {
                    previous_button.set_visible(false);
                }
            }));

        let phone_number_entry = &*self_.phone_number_entry;
        let confirmation_code_entry = &*self_.confirmation_code_entry;
        self_.next_button
            .connect_clicked(glib::clone!(@weak content_leaflet, @weak phone_number_entry, @weak confirmation_code_entry, @strong gtk_sender => move |_| {
                let page_name = content_leaflet.visible_child_name().unwrap();
                if page_name == "phone_number_page" {
                    telegram::send_gtk_event(&gtk_sender,
                        telegram::GtkEvent::SendPhoneNumber(
                            phone_number_entry.text().to_string()));
                } else if page_name == "confirmation_code_page" {
                    telegram::send_gtk_event(&gtk_sender,
                        telegram::GtkEvent::SendConfirmationCode(
                            confirmation_code_entry.text().to_string()));
                }
            }));
    }

    pub fn navigate_forward(&self) {
        let self_ = imp::AddAccountWindow::from_instance(self);
        self_.content_leaflet.navigate(adw::NavigationDirection::Forward);
        self_.previous_button.set_visible(true);
    }

    pub fn show_phone_number_error(&self, error: AuthorizationError) {
        let self_ = imp::AddAccountWindow::from_instance(self);
        self_.phone_number_error_label.set_text(&error.to_string());
    }

    pub fn show_confirmation_code_error(&self, error: SignInError) {
        let self_ = imp::AddAccountWindow::from_instance(self);
        self_.confirmation_code_error_label.set_text(&error.to_string());
    }
}
