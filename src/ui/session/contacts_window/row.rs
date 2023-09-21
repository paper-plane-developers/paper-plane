use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::strings;
use crate::ui;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Row)]
    #[template(resource = "/app/drey/paper-plane/ui/session/contacts_window/row.ui")]
    pub(crate) struct Row {
        #[property(get, set = Self::set_user)]
        pub(super) user: glib::WeakRef<model::User>,
        #[template_child]
        pub(super) avatar: TemplateChild<ui::Avatar>,
        #[template_child]
        pub(super) labels_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) name_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) status_label: TemplateChild<gtk::Inscription>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "PaplContactRow";
        type Type = super::Row;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("contactrow");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for Row {}

    impl Row {
        fn set_user(&self, user: Option<&model::User>) {
            if let Some(user) = user {
                // TODO: Make these strings auto update when needed
                let name = strings::user_display_name(user, true);
                self.name_label.set_text(Some(&name));

                let status = strings::user_status(&user.status().0);
                self.status_label.set_text(Some(&status));
            }

            self.user.set(user);
        }
    }
}

glib::wrapper! {
    pub(crate) struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}
