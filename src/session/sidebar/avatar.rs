use super::Sidebar;

use glib::closure;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, gsk, CompositeTemplate};
use tdgrand::enums::{UserStatus, UserType};

use crate::session::user::BoxedUserStatus;
use crate::session::{Chat, Session, User};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    use crate::session::components::Avatar as ComponentsAvatar;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-avatar.ui")]
    pub struct Avatar {
        /// A `Chat` or `User`
        pub item: RefCell<Option<glib::Object>>,
        pub binding: RefCell<Option<gtk::ExpressionWatch>>,
        pub is_online: Cell<bool>,
        // The first Option indicates whether we've once tried to compile the shader. The second
        // Option contains the compiled shader.
        pub mask_shader: RefCell<Option<Option<gsk::GLShader>>>,
        #[template_child]
        pub avatar: TemplateChild<ComponentsAvatar>,
        #[template_child]
        pub online_indicator_mask: TemplateChild<adw::Bin>,
        #[template_child]
        pub online_indicator_dot: TemplateChild<adw::Bin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Avatar {
        const NAME: &'static str = "SidebarAvatar";
        type Type = super::Avatar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Avatar {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "item",
                        "Item",
                        "The item of this avatar",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-online",
                        "Is Online",
                        "Whether this SidebarAvatar's user is online",
                        false,
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
                "is-online" => obj.set_is_online(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                "is-online" => obj.is_online().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.avatar.unparent();
            self.online_indicator_mask.unparent();
            self.online_indicator_dot.unparent();
        }
    }

    impl WidgetImpl for Avatar {
        // Inspired by
        // https://gitlab.gnome.org/GNOME/libadwaita/-/blob/1171616701bf12a1c56bbad3f0e8821208d87029/src/adw-indicator-bin.c
        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
            if !widget.is_online() {
                widget.snapshot_child(&*self.avatar, snapshot);
                return;
            }

            let child_snapshot = gtk::Snapshot::new();
            widget.snapshot_child(&*self.avatar, &child_snapshot);
            let child_node = child_snapshot.to_node().unwrap();

            widget.ensure_mask_shader();

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
                widget.snapshot_child(&*self.online_indicator_mask, snapshot);
                snapshot.gl_shader_pop_texture();

                snapshot.pop();
            } else {
                widget.snapshot_child(&*self.online_indicator_mask, snapshot);
            }

            widget.snapshot_child(&*self.online_indicator_dot, snapshot);
        }
    }
}

glib::wrapper! {
    pub struct Avatar(ObjectSubclass<imp::Avatar>)
        @extends gtk::Widget;
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Avatar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SidebarAvatar")
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

    pub fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().to_owned()
    }

    pub fn set_item(&self, item: Option<glib::Object>) {
        if self.item() == item {
            return;
        }

        let imp = self.imp();

        if let Some(binding) = imp.binding.take() {
            binding.unwatch();
        }

        if let Some(ref item) = item {
            if let Some(chat) = item.downcast_ref::<Chat>() {
                imp.avatar.set_item(Some(chat.avatar().to_owned()));

                match chat.type_().user() {
                    Some(user) => self.setup_is_online_binding(user),
                    None => self.set_is_online(false),
                }
            } else if let Some(user) = item.downcast_ref::<User>() {
                imp.avatar.set_item(Some(user.avatar().to_owned()));

                self.setup_is_online_binding(user);
            } else {
                unreachable!("Unexpected item type: {:?}", item);
            }
        }

        imp.item.replace(item);
        self.notify("item");
    }

    pub fn is_online(&self) -> bool {
        self.imp().is_online.get()
    }

    pub fn set_is_online(&self, is_online: bool) {
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
