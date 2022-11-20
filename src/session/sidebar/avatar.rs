use super::Sidebar;

use glib::closure;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, gsk, CompositeTemplate};
use tdlib::enums::{UserStatus, UserType};

use crate::tdlib::{BoxedUserStatus, Chat, User};
use crate::Session;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    use crate::session::components::Avatar as ComponentsAvatar;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-avatar.ui")]
    pub(crate) struct Avatar {
        /// A `Chat` or `User`
        pub(super) item: RefCell<Option<glib::Object>>,
        pub(super) binding: RefCell<Option<gtk::ExpressionWatch>>,
        pub(super) is_online: Cell<bool>,
        // The first Option indicates whether we've once tried to compile the shader. The second
        // Option contains the compiled shader.
        pub(super) mask_shader: RefCell<Option<Option<gsk::GLShader>>>,
        #[template_child]
        pub(super) avatar: TemplateChild<ComponentsAvatar>,
        #[template_child]
        pub(super) online_indicator_mask: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) online_indicator_dot: TemplateChild<adw::Bin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Avatar {
        const NAME: &'static str = "SidebarAvatar";
        type Type = super::Avatar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Avatar {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<glib::Object>("item")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-online")
                        .explicit_notify()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.set_item(value.get().unwrap()),
                "is-online" => obj.set_is_online(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.item().to_value(),
                "is-online" => obj.is_online().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            self.avatar.unparent();
            self.online_indicator_mask.unparent();
            self.online_indicator_dot.unparent();
        }
    }

    impl WidgetImpl for Avatar {
        // Inspired by
        // https://gitlab.gnome.org/GNOME/libadwaita/-/blob/1171616701bf12a1c56bbad3f0e8821208d87029/src/adw-indicator-bin.c
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            if !obj.is_online() {
                obj.snapshot_child(&*self.avatar, snapshot);
                return;
            }

            let child_snapshot = gtk::Snapshot::new();
            obj.snapshot_child(&*self.avatar, &child_snapshot);
            let child_node = child_snapshot.to_node().unwrap();

            obj.ensure_mask_shader();

            let maybe_compiled_masked_shader = self.mask_shader.borrow().clone().flatten();

            if let Some(ref compiled_mask_shader) = maybe_compiled_masked_shader {
                snapshot.push_gl_shader(
                    compiled_mask_shader,
                    &child_node.bounds(),
                    &gsk::ShaderArgsBuilder::new(compiled_mask_shader, None).to_args(),
                );
            }

            snapshot.append_node(&child_node);

            if maybe_compiled_masked_shader.is_some() {
                snapshot.gl_shader_pop_texture();
                obj.snapshot_child(&*self.online_indicator_mask, snapshot);
                snapshot.gl_shader_pop_texture();

                snapshot.pop();
            } else {
                obj.snapshot_child(&*self.online_indicator_mask, snapshot);
            }

            obj.snapshot_child(&*self.online_indicator_dot, snapshot);
        }
    }
}

glib::wrapper! {
    pub(crate) struct Avatar(ObjectSubclass<imp::Avatar>)
        @extends gtk::Widget;
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Avatar {
    pub(crate) fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_is_online_binding(&self, user: &User) {
        if !matches!(user.type_().0, UserType::Regular) {
            self.set_is_online(false);
            return;
        }

        // This should never panic as there must always be a `Sidebar` as ancestor having a
        // `Session`.
        let session = self
            .ancestor(Sidebar::static_type())
            .unwrap()
            .downcast_ref::<Sidebar>()
            .unwrap()
            .session()
            .unwrap();

        let my_id = session.me().id();
        let user_id = user.id();
        let is_online_binding = gtk::ObjectExpression::new(user)
            .chain_property::<User>("status")
            .chain_closure::<bool>(closure!(
                |_: Session, interlocutor_status: BoxedUserStatus| {
                    matches!(interlocutor_status.0, UserStatus::Online(_)) && my_id != user_id
                }
            ))
            .bind(self, "is-online", Some(&session));

        self.imp().binding.replace(Some(is_online_binding));
    }

    pub(crate) fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().to_owned()
    }

    pub(crate) fn set_item(&self, item: Option<glib::Object>) {
        if self.item() == item {
            return;
        }

        let imp = self.imp();

        if let Some(binding) = imp.binding.take() {
            binding.unwatch();
        }

        self.imp().avatar.set_item(item.clone());

        if let Some(ref item) = item {
            if let Some(chat) = item.downcast_ref::<Chat>() {
                match chat.type_().user() {
                    Some(user) => self.setup_is_online_binding(user),
                    None => self.set_is_online(false),
                }
            } else if let Some(user) = item.downcast_ref::<User>() {
                self.setup_is_online_binding(user);
            } else {
                unreachable!("Unexpected item type: {:?}", item);
            }
        }

        imp.item.replace(item);
        self.notify("item");
    }

    pub(crate) fn is_online(&self) -> bool {
        self.imp().is_online.get()
    }

    pub(crate) fn set_is_online(&self, is_online: bool) {
        if self.is_online() == is_online {
            return;
        }
        self.imp().is_online.set(is_online);
        self.notify("is-online");
    }

    // Inspired by
    // https://gitlab.gnome.org/GNOME/libadwaita/-/blob/1171616701bf12a1c56bbad3f0e8821208d87029/src/adw-indicator-bin.c
    fn ensure_mask_shader(&self) {
        let imp = self.imp();

        if imp.mask_shader.borrow().is_some() {
            // We've already tried to compile the shader before.
            return;
        }

        let shader = gsk::GLShader::from_resource("/org/gnome/Adwaita/glsl/mask.glsl");
        let renderer = self.native().unwrap().renderer();
        let compiled_shader = match shader.compile(&renderer) {
            Err(e) => {
                // If shaders aren't supported, the error doesn't matter and we just silently fall
                // back.
                log::error!("Couldn't compile shader: {}", e);
                None
            }
            Ok(_) => Some(shader),
        };

        imp.mask_shader.replace(Some(compiled_shader));
    }
}
