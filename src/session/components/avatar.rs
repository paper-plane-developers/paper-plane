use glib::closure;
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
    pub(crate) struct Avatar {
        pub(super) item: RefCell<Option<AvatarItem>>,
        #[template_child]
        pub(super) avatar: TemplateChild<adw::Avatar>,
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
                    glib::ParamSpecObject::new(
                        "item",
                        "Item",
                        "The avatar item displayed by this widget",
                        AvatarItem::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecInt::new(
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
                "size" => obj.set_size(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                "size" => obj.size().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_expressions();
        }
    }

    impl WidgetImpl for Avatar {}
    impl BinImpl for Avatar {}
}

glib::wrapper! {
    pub(crate) struct Avatar(ObjectSubclass<imp::Avatar>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Avatar {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ComponentsAvatar")
    }

    fn setup_expressions(&self) {
        let item_expression = Self::this_expression("item");

        let icon_name_expression = item_expression.chain_property::<AvatarItem>("icon-name");

        let custom_image_expression = item_expression.chain_property::<AvatarItem>("image");
        let custom_image_expression =
            gtk::ClosureExpression::new::<Option<gtk::gdk::Paintable>, _, _>(
                &[
                    icon_name_expression.clone().upcast(),
                    custom_image_expression.upcast(),
                ],
                closure!(|_: Self,
                          icon_name: Option<String>,
                          cusom_image: Option<gtk::gdk::Paintable>| {
                    match icon_name {
                        Some(_) => None,
                        None => cusom_image,
                    }
                }),
            );

        let show_initials_expression = icon_name_expression.chain_closure::<bool>(closure!(
            |_: Self, icon_name: Option<String>| icon_name.is_none()
        ));

        let text_expression = item_expression.chain_property::<AvatarItem>("display-name");
        let text_expression = gtk::ClosureExpression::new::<Option<String>, _, _>(
            &[
                icon_name_expression.clone().upcast(),
                text_expression.upcast(),
            ],
            closure!(
                |_: Self, icon_name: Option<String>, display_name: Option<String>| {
                    icon_name
                        // If we use an icon for the avatar, we always want its color to be blue. As
                        // AdwAvatar doesn't allow us to set the color in an explicit manner, we are
                        // forced to use this workaround.
                        .map(|_| Some(String::from("-")))
                        .unwrap_or(display_name)
                }
            ),
        );

        let imp = self.imp();

        icon_name_expression.bind(&*imp.avatar, "icon-name", Some(self));
        custom_image_expression.bind(&*imp.avatar, "custom-image", Some(self));
        show_initials_expression.bind(&*imp.avatar, "show-initials", Some(self));
        text_expression.bind(&*imp.avatar, "text", Some(self));
    }

    fn request_avatar_image(&self) {
        if let Some(item) = &*self.imp().item.borrow() {
            item.set_needed(true);
        }
    }

    pub(crate) fn item(&self) -> Option<AvatarItem> {
        self.imp().item.borrow().clone()
    }

    pub(crate) fn set_item(&self, item: Option<AvatarItem>) {
        self.imp().item.replace(item);

        self.request_avatar_image();

        self.notify("item");
    }

    pub(crate) fn size(&self) -> i32 {
        self.imp().avatar.size()
    }

    pub(crate) fn set_size(&self, size: i32) {
        self.imp().avatar.set_size(size);
        self.notify("size");
    }
}
