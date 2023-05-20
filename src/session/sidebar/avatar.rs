use std::cell::Cell;
use std::cell::RefCell;

use glib::closure;
use gtk::glib;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use tdlib::enums::UserStatus;
use tdlib::enums::UserType;

use super::Sidebar;
use crate::components::Avatar as ComponentsAvatar;
use crate::tdlib::BoxedUserStatus;
use crate::tdlib::Chat;
use crate::tdlib::User;
use crate::Session;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/sidebar-avatar.ui")]
    pub(crate) struct Avatar {
        /// A `Chat` or `User`
        pub(super) item: RefCell<Option<glib::Object>>,
        pub(super) binding: RefCell<Option<gtk::ExpressionWatch>>,
        pub(super) is_online: Cell<bool>,
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
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            if !obj.is_online() {
                obj.snapshot_child(&*self.avatar, snapshot);
                return;
            }

            snapshot.push_mask(gsk::MaskMode::InvertedAlpha);

            obj.snapshot_child(&*self.online_indicator_mask, snapshot);
            snapshot.pop();

            obj.snapshot_child(&*self.avatar, snapshot);
            snapshot.pop();

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
        glib::Object::new()
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
}
