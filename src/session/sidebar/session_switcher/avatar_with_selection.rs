use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::session::components::Avatar;
use crate::session::Avatar as AvatarItem;

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/avatar-with-selection.ui")]
    pub struct AvatarWithSelection {
        #[template_child]
        pub child_avatar: TemplateChild<Avatar>,
        #[template_child]
        pub checkmark: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AvatarWithSelection {
        const NAME: &'static str = "AvatarWithSelection";
        type Type = super::AvatarWithSelection;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Avatar::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AvatarWithSelection {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "item",
                        "Item",
                        "The Avatar item displayed by this widget",
                        AvatarItem::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_int(
                        "size",
                        "Size",
                        "The size of the Avatar",
                        -1,
                        i32::MAX,
                        -1,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "selected",
                        "Selected",
                        "Style helper for the inner Avatar",
                        false,
                        glib::ParamFlags::WRITABLE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "item" => self.child_avatar.set_item(value.get().unwrap()),
                "size" => self.child_avatar.set_size(value.get().unwrap()),
                "selected" => obj.set_selected(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
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
    pub struct AvatarWithSelection(ObjectSubclass<imp::AvatarWithSelection>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl Default for AvatarWithSelection {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarWithSelection {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AvatarWithSelection")
    }

    pub fn set_selected(&self, selected: bool) {
        let self_ = imp::AvatarWithSelection::from_instance(self);

        self_.checkmark.set_visible(selected);

        if selected {
            self_.child_avatar.add_css_class("selected-avatar");
        } else {
            self_.child_avatar.remove_css_class("selected-avatar");
        }
    }

    pub fn avatar(&self) -> &Avatar {
        &imp::AvatarWithSelection::from_instance(self).child_avatar
    }
}
