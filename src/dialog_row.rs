use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

use crate::dialog_data::DialogData;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/dialog_row.ui")]
    pub struct DialogRow {
        #[template_child]
        pub chat_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub last_message_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DialogRow {
        const NAME: &'static str = "DialogRow";
        type Type = super::DialogRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DialogRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for DialogRow {}
    impl ListBoxRowImpl for DialogRow {}
}

glib::wrapper! {
    pub struct DialogRow(ObjectSubclass<imp::DialogRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl DialogRow {
    pub fn new(data: &DialogData) -> Self {
        let dialog_row = glib::Object::new(&[])
            .expect("Failed to create DialogRow");

        let self_ = imp::DialogRow::from_instance(&dialog_row);

        data.bind_property("chat-name", &self_.chat_name_label.get(), "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        data.bind_property("last-message", &self_.last_message_label.get(), "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        dialog_row
    }
}
