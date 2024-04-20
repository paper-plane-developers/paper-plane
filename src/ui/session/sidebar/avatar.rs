use std::cell::Cell;
use std::cell::RefCell;
use std::sync::OnceLock;

use glib::closure;
use gtk::glib;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/avatar.ui")]
    pub(crate) struct Avatar {
        /// A `Chat` or `User`
        pub(super) item: RefCell<Option<glib::Object>>,
        pub(super) binding: RefCell<Option<gtk::ExpressionWatch>>,
        pub(super) is_online: Cell<bool>,
        #[template_child]
        pub(super) avatar: TemplateChild<ui::Avatar>,
        #[template_child]
        pub(super) online_indicator_mask: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) online_indicator_dot: TemplateChild<adw::Bin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Avatar {
        const NAME: &'static str = "PaplSidebarAvatar";
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
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<glib::Object>("item")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("is-online")
                        .explicit_notify()
                        .build(),
                ]
            })
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

    fn setup_is_online_binding(&self, user: &model::User) {
        if !matches!(user.user_type().0, tdlib::enums::UserType::Regular) {
            self.set_is_online(false);
            return;
        }

        let session = user.session_();
        let my_id = session.me_().id();
        let user_id = user.id();
        let is_online_binding = gtk::ObjectExpression::new(user)
            .chain_property::<model::User>("status")
            .chain_closure::<bool>(closure!(
                |_: Option<model::ClientStateSession>,
                 interlocutor_status: model::BoxedUserStatus| {
                    matches!(interlocutor_status.0, tdlib::enums::UserStatus::Online(_))
                        && my_id != user_id
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

        if let Some(ref item) = item {
            if let Some(chat) = item.downcast_ref::<model::Chat>() {
                if chat.is_own_chat() {
                    imp.avatar.set_item(Some(item.clone()));
                } else {
                    match chat.chat_type().user() {
                        Some(user) => {
                            self.setup_is_online_binding(&user);
                            imp.avatar.set_item(Some(user.clone().upcast()));
                        }
                        None => {
                            self.set_is_online(false);
                            imp.avatar.set_item(Some(item.clone()));
                        }
                    }
                }
            } else if let Some(user) = item.downcast_ref::<model::User>() {
                self.setup_is_online_binding(user);
                imp.avatar.set_item(Some(user.clone().upcast()));
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
