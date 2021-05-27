use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/login.ui")]
    pub struct Login {
        pub client_id: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for Login {}
    impl BinImpl for Login {}
}

glib::wrapper! {
    pub struct Login(ObjectSubclass<imp::Login>)
        @extends gtk::Widget, adw::Bin;
}

impl Login {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Login")
    }

    pub fn set_client_id(&self, client_id: i32) {
        let priv_ = imp::Login::from_instance(self);
        priv_.client_id.set(client_id);
    }
}
