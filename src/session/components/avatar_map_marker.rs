use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::tdlib::User;

mod imp {
    use super::*;
    use glib::clone;

    use crate::session::components::Avatar;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="ComponentsAvatarMapMarker" parent="GtkWidget">
        <property name="layout-manager">
          <object class="GtkBinLayout"/>
        </property>
        <child>
          <object class="GtkOverlay">
            <child>
              <object class="GtkImage">
                <style>
                  <class name="hull"/>
                </style>
                <property name="icon-name">avatar-map-marker-hull-symbolic</property>
                <property name="pixel-size">68</property>
                <style>
                  <class name="icon-dropshadow"/>
                </style>
              </object>
            </child>
            <child type="overlay">
              <object class="ComponentsAvatar" id="avatar">
                <property name="size">48</property>
                <property name="valign">start</property>
                <property name="margin-top">4</property>
              </object>
            </child>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    pub(crate) struct AvatarMapMarker {
        #[template_child]
        pub(super) avatar: TemplateChild<Avatar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AvatarMapMarker {
        const NAME: &'static str = "ComponentsAvatarMapMarker";
        type Type = super::AvatarMapMarker;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_css_name("avatarmapmarker");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AvatarMapMarker {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "user",
                    "User",
                    "The User of the avatar map marker",
                    User::static_type(),
                    glib::ParamFlags::READWRITE
                        | glib::ParamFlags::CONSTRUCT
                        | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                "user" => obj.set_user(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "user" => obj.user().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.avatar.connect_notify_local(
                Some("item"),
                clone!(@weak obj => move |_, _| {
                    obj.notify("user");
                }),
            );
        }

        fn dispose(&self, obj: &Self::Type) {
            let mut child = obj.first_child();
            while let Some(child_) = child {
                child = child_.next_sibling();
                child_.unparent();
            }
        }
    }

    impl WidgetImpl for AvatarMapMarker {}
}

glib::wrapper! {
    pub(crate) struct AvatarMapMarker(ObjectSubclass<imp::AvatarMapMarker>)
        @extends gtk::Widget;
}

impl From<&User> for AvatarMapMarker {
    fn from(user: &User) -> Self {
        glib::Object::new(&[("user", &user)]).expect("Failed to create ComponentsAvatarMapMarker")
    }
}

impl AvatarMapMarker {
    pub(crate) fn user(&self) -> Option<User> {
        self.imp()
            .avatar
            .item()
            .map(|item| item.downcast().unwrap())
    }

    pub(crate) fn set_user(&self, user: User) {
        if self.user().as_ref() == Some(&user) {
            return;
        }
        self.imp().avatar.set_item(Some(user.upcast()));
    }
}
