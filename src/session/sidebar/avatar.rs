use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, gsk};
use tdgrand::enums::{ChatType, UserStatus, UserType};

use crate::session::user::BoxedUserStatus;
use crate::session::{Chat, Session, User};

mod imp {

    use super::*;
    use gtk::CompositeTemplate;
    use std::cell::{Cell, RefCell};

    use crate::session::components::Avatar as ComponentsAvatar;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-avatar.ui")]
    pub struct Avatar {
        pub chat: RefCell<Option<Chat>>,
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
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "chat",
                        "Chat",
                        "The chat represented by this SidebarAvatar",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boolean(
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
                "chat" => obj.set_chat(value.get().unwrap()),
                "is-online" => obj.set_is_online(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "chat" => obj.chat().to_value(),
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
            let child_node = child_snapshot.free_to_node().unwrap();

            widget.ensure_mask_shader();

            let maybe_compiled_masked_shader = self.mask_shader.borrow().clone().flatten();

            if let Some(ref compiled_mask_shader) = maybe_compiled_masked_shader {
                snapshot.push_gl_shader(
                    compiled_mask_shader,
                    &child_node.bounds(),
                    gsk::ShaderArgsBuilder::new(compiled_mask_shader, None)
                        .to_args()
                        .as_ref()
                        .unwrap(),
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

    pub fn chat(&self) -> Option<Chat> {
        let self_ = imp::Avatar::from_instance(self);
        self_.chat.borrow().clone()
    }
    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let self_ = imp::Avatar::from_instance(self);

        if let Some(ref chat) = chat {
            if let Some(interlocutor_id) = interlocutor_id(chat) {
                let interlocutor = chat
                    .session()
                    .user_list()
                    .get_or_create_user(interlocutor_id);

                if let UserType::Regular = interlocutor.type_().0 {
                    let session_expression = gtk::ConstantExpression::new(&chat.session());
                    let me_expression = gtk::PropertyExpression::new(
                        Session::static_type(),
                        Some(&session_expression),
                        "me",
                    );

                    let interlocutor_expression = gtk::ConstantExpression::new(&interlocutor);
                    let interlocutor_status_expression =
                        User::status_expression(&interlocutor_expression);

                    let interlocutor_is_online_expression = gtk::ClosureExpression::new(
                        move |expressions| -> bool {
                            let maybe_me = expressions[1].get::<User>();
                            maybe_me
                                .map(|me| {
                                    let user_status =
                                        expressions[2].get::<BoxedUserStatus>().unwrap().0;
                                    matches!(user_status, UserStatus::Online(_))
                                        && me.id() != interlocutor_id
                                })
                                .unwrap_or_default()
                        },
                        &[me_expression.upcast(), interlocutor_status_expression],
                    );
                    interlocutor_is_online_expression.bind(self, "is-online", gtk::NONE_WIDGET);
                }
            }
        }

        self_.chat.replace(chat);
        self.notify("chat");
    }

    pub fn is_online(&self) -> bool {
        let self_ = imp::Avatar::from_instance(self);
        self_.is_online.get()
    }

    pub fn set_is_online(&self, is_online: bool) {
        if self.is_online() == is_online {
            return;
        }
        let self_ = imp::Avatar::from_instance(self);
        self_.is_online.set(is_online);

        self.notify("is-online");
    }

    // Inspired by
    // https://gitlab.gnome.org/GNOME/libadwaita/-/blob/1171616701bf12a1c56bbad3f0e8821208d87029/src/adw-indicator-bin.c
    fn ensure_mask_shader(&self) {
        let self_ = imp::Avatar::from_instance(self);

        if self_.mask_shader.borrow().is_some() {
            // We've already tried to compile the shader before.
            return;
        }

        let native = self.native().unwrap();
        let renderer = native.renderer().unwrap();

        let shader = gsk::GLShader::from_resource("/org/gnome/Adwaita/glsl/mask.glsl");
        let compiled_shader = match shader.compile(&renderer) {
            Err(e) => {
                // If shaders aren't supported, the error doesn't matter and we just silently fall
                // back.
                log::error!("Couldn't compile shader: {}", e);
                None
            }
            Ok(_) => Some(shader),
        };

        self_.mask_shader.replace(Some(compiled_shader));
    }
}

fn interlocutor_id(chat: &Chat) -> Option<i32> {
    match chat.type_() {
        ChatType::Private(private) => Some(private.user_id),
        ChatType::Secret(secret) => Some(secret.user_id),
        _ => None,
    }
}
