use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use glib::Properties;
    use std::cell::RefCell;

    use crate::components::Avatar;
    use crate::strings;
    use crate::tdlib::User;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::ContactRow)]
    #[template(string = r#"
    template ContactRow {
        .ComponentsAvatar avatar {
            size: 32;
            item: bind ContactRow.user;
        }

        Box labels_box {
            orientation: vertical;
            homogeneous: true;
            hexpand: true;

            Inscription name_label {
                text-overflow: ellipsize_end;
            }

            Inscription status_label {
                text-overflow: ellipsize_end;

                styles [
                    "dim-label",
                    "small-body",
                ]
            }
        }
    }
    "#)]
    pub(crate) struct ContactRow {
        #[property(get, set = Self::set_user)]
        pub(super) user: RefCell<Option<User>>,
        #[template_child]
        pub(super) avatar: TemplateChild<Avatar>,
        #[template_child]
        pub(super) labels_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) name_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) status_label: TemplateChild<gtk::Inscription>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContactRow {
        const NAME: &'static str = "ContactRow";
        type Type = super::ContactRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_css_name("contactrow");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContactRow {
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

    impl WidgetImpl for ContactRow {}

    impl ContactRow {
        fn set_user(&self, user: Option<User>) {
            if let Some(ref user) = user {
                // TODO: Make these strings auto update when needed
                let name = strings::user_display_name(user, true);
                self.name_label.set_text(Some(&name));

                let status = strings::user_status(&user.status().0);
                self.status_label.set_text(Some(&status));
            }

            self.user.replace(user);
        }
    }
}

glib::wrapper! {
    pub(crate) struct ContactRow(ObjectSubclass<imp::ContactRow>)
        @extends gtk::Widget;
}
