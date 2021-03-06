use adw::NavigationDirection;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use std::sync::mpsc;

use crate::telegram;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use glib::subclass;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/add_account_window.ui")]
    pub struct AddAccountWindow {
        #[template_child]
        pub content_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub phone_number_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub phone_number_next: TemplateChild<gtk::Button>,
        #[template_child]
        pub confirmation_code_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub confirmation_code_next: TemplateChild<gtk::Button>,
    }

    impl ObjectSubclass for AddAccountWindow {
        const NAME: &'static str = "AddAccountWindow";
        type Type = super::AddAccountWindow;
        type ParentType = adw::Window;
        type Interfaces = ();
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                content_leaflet: TemplateChild::default(),
                phone_number_entry: TemplateChild::default(),
                phone_number_next: TemplateChild::default(),
                confirmation_code_entry: TemplateChild::default(),
                confirmation_code_next: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
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

    pub fn init_signals(&self, tg_sender: &mpsc::Sender<telegram::EventTG>) {
        let self_ = imp::AddAccountWindow::from_instance(self);

        let phone_number_entry = &*self_.phone_number_entry;
        let tg_sender_clone = tg_sender.clone();
        self_.phone_number_next
            .connect_clicked(glib::clone!(@weak phone_number_entry => move |_| {
                tg_sender_clone.send(telegram::EventTG::SendPhoneNumber(
                    phone_number_entry.get_text().to_string())).unwrap();
            }));

        let confirmation_code_entry = &*self_.confirmation_code_entry;
        let tg_sender_clone = tg_sender.clone();
        self_.confirmation_code_next
            .connect_clicked(glib::clone!(@weak confirmation_code_entry => move |_| {
                tg_sender_clone.send(telegram::EventTG::SendConfirmationCode(
                    confirmation_code_entry.get_text().to_string())).unwrap();
            }));
    }

    pub fn navigate_forward(&self) {
        let self_ = imp::AddAccountWindow::from_instance(self);
        self_.content_leaflet.navigate(NavigationDirection::Forward);
    }
}
