use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::session::Avatar as AvatarItem;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/components-avatar.ui")]
    pub struct Avatar {
        pub item: RefCell<Option<AvatarItem>>,
        pub display_name: RefCell<Option<String>>,
        #[template_child]
        pub avatar: TemplateChild<adw::Avatar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Avatar {
        const NAME: &'static str = "ComponentsAvatar";
        type Type = super::Avatar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Avatar {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "item",
                        "Item",
                        "The avatar item displayed by this widget",
                        AvatarItem::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_string(
                        "display-name",
                        "Display Name",
                        "The display name used for this avatar",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_int(
                        "size",
                        "Size",
                        "The size of this avatar",
                        -1,
                        i32::MAX,
                        -1,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "item" => obj.set_item(value.get().unwrap()),
                "display-name" => obj.set_display_name(value.get().unwrap()),
                "size" => obj.set_size(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                "display-name" => obj.display_name().to_value(),
                "size" => obj.size().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Avatar {}
    impl BinImpl for Avatar {}
}

glib::wrapper! {
    pub struct Avatar(ObjectSubclass<imp::Avatar>)
        @extends gtk::Widget, adw::Bin;
}

impl Avatar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ComponentsAvatar")
    }

    fn request_avatar_image(&self) {
        let self_ = imp::Avatar::from_instance(self);
        if let Some(item) = &*self_.item.borrow() {
            item.set_needed(true);
        }
    }

    pub fn item(&self) -> Option<AvatarItem> {
        let self_ = imp::Avatar::from_instance(self);
        self_.item.borrow().clone()
    }

    pub fn set_item(&self, item: Option<AvatarItem>) {
        let self_ = imp::Avatar::from_instance(self);
        self_.item.replace(item);

        self.request_avatar_image();

        self.notify("item");
    }

    pub fn display_name(&self) -> Option<String> {
        let self_ = imp::Avatar::from_instance(self);
        self_.display_name.borrow().clone()
    }

    pub fn set_display_name(&self, display_name: Option<String>) {
        let self_ = imp::Avatar::from_instance(self);
        self_.display_name.replace(display_name);

        self.notify("display-name");
    }

    pub fn size(&self) -> i32 {
        let self_ = imp::Avatar::from_instance(self);
        self_.avatar.size()
    }

    pub fn set_size(&self, size: i32) {
        let self_ = imp::Avatar::from_instance(self);
        self_.avatar.set_size(size);

        self.notify("size");
    }
}
