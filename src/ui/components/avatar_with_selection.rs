use std::sync::OnceLock;

use adw::subclass::prelude::*;
use glib::subclass::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::CompositeTemplate;

use crate::ui;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/components/avatar_with_selection.ui")]
    pub(crate) struct AvatarWithSelection {
        #[template_child]
        pub(super) child_avatar: TemplateChild<ui::Avatar>,
        #[template_child]
        pub(super) checkmark: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AvatarWithSelection {
        const NAME: &'static str = "PaplAvatarWithSelection";
        type Type = super::AvatarWithSelection;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AvatarWithSelection {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<glib::Object>("item")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecInt::builder("size").build(),
                    glib::ParamSpecBoolean::builder("selected")
                        .write_only()
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "item" => self.child_avatar.set_item(value.get().unwrap()),
                "size" => self.child_avatar.set_size(value.get().unwrap()),
                "selected" => obj.set_selected(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => self.child_avatar.item().to_value(),
                "size" => self.child_avatar.size().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for AvatarWithSelection {}
    impl BinImpl for AvatarWithSelection {}
}

glib::wrapper! {
    /// A widget displaying an `Avatar` for an `Account`.
    pub(crate) struct AvatarWithSelection(ObjectSubclass<imp::AvatarWithSelection>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl Default for AvatarWithSelection {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarWithSelection {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn set_selected(&self, selected: bool) {
        let imp = self.imp();
        imp.checkmark.set_visible(selected);

        if selected {
            imp.child_avatar.add_css_class("selected-avatar");
        } else {
            imp.child_avatar.remove_css_class("selected-avatar");
        }
    }
}
