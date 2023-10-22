use std::cell::Cell;

use glib::Properties;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::StyleVariantPreview)]
    #[template(resource = "/app/drey/paper-plane/ui/components/style_variant_preview.ui")]
    pub(crate) struct StyleVariantPreview {
        #[property(get, set, builder(adw::ColorScheme::Default))]
        pub(super) color_scheme: Cell<adw::ColorScheme>,
        #[template_child]
        pub(super) picture: TemplateChild<gtk::Picture>,
    }

    impl Default for StyleVariantPreview {
        fn default() -> Self {
            Self {
                color_scheme: Cell::new(adw::ColorScheme::Default),
                picture: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StyleVariantPreview {
        const NAME: &'static str = "StyleVariantPreview";
        type Type = super::StyleVariantPreview;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("stylevariantpreview");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StyleVariantPreview {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_color_scheme_notify(|obj| {
                let texture = gdk::Texture::from_resource(match obj.color_scheme() {
                    adw::ColorScheme::PreferLight | adw::ColorScheme::ForceLight => {
                        "/app/drey/paper-plane/assets/preview-light.svg"
                    }
                    adw::ColorScheme::PreferDark | adw::ColorScheme::ForceDark => {
                        "/app/drey/paper-plane/assets/preview-dark.svg"
                    }
                    _ => "/app/drey/paper-plane/assets/preview-system.svg",
                });
                obj.imp().picture.set_paintable(Some(&texture));
            });
        }

        fn dispose(&self) {
            let mut child = self.obj().first_child();
            while let Some(child_) = child {
                child = child_.next_sibling();
                child_.unparent();
            }
        }
    }

    impl WidgetImpl for StyleVariantPreview {}
}

glib::wrapper! {
    pub(crate) struct StyleVariantPreview(ObjectSubclass<imp::StyleVariantPreview>) @extends gtk::Widget;
}
