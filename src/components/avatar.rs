use std::cell::OnceCell;
use std::cell::RefCell;

use adw::subclass::prelude::BinImpl;
use glib::clone;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use tdlib::enums::UserType;

use crate::strings;
use crate::tdlib::Avatar as AvatarItem;
use crate::tdlib::Chat;
use crate::tdlib::User;
use crate::utils::spawn;
use crate::Session;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/components-avatar.ui")]
    pub(crate) struct Avatar {
        /// A `Chat` or `User`
        pub(super) item: RefCell<Option<glib::Object>>,
        pub(super) user_signal_group: OnceCell<glib::SignalGroup>,
        pub(super) chat_signal_group: OnceCell<glib::SignalGroup>,
        #[template_child]
        pub(super) avatar: TemplateChild<adw::Avatar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Avatar {
        const NAME: &'static str = "ComponentsAvatar";
        type Type = super::Avatar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("avatar");
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
                    glib::ParamSpecObject::builder::<glib::Object>("item")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecString::builder("custom-text")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecInt::builder("size")
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
                "custom-text" => obj.set_custom_text(value.get().unwrap()),
                "size" => obj.set_size(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.item().to_value(),
                "custom-text" => obj.custom_text().to_value(),
                "size" => obj.size().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().create_signal_groups();
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
        glib::Object::new()
    }

    fn create_signal_groups(&self) {
        let imp = self.imp();

        let user_signal_group = glib::SignalGroup::new(User::static_type());
        user_signal_group.connect_notify_local(
            Some("type"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_display_name();
                obj.update_avatar();
            }),
        );
        user_signal_group.connect_notify_local(
            Some("first-name"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_display_name();
            }),
        );
        user_signal_group.connect_notify_local(
            Some("last-name"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_display_name();
            }),
        );
        user_signal_group.connect_notify_local(
            Some("avatar"),
            clone!(@weak self as obj => move|_, _| {
                obj.update_avatar();
            }),
        );
        imp.user_signal_group.set(user_signal_group).unwrap();

        let chat_signal_group = glib::SignalGroup::new(Chat::static_type());
        chat_signal_group.connect_notify_local(
            Some("title"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_display_name()
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("avatar"),
            clone!(@weak self as obj => move|_, _| {
                obj.update_avatar();
            }),
        );

        imp.chat_signal_group.set(chat_signal_group).unwrap();
    }

    fn load_image(&self, avatar_item: Option<AvatarItem>, session: Session) {
        if let Some(avatar_item) = avatar_item {
            let file = avatar_item.0;
            if file.local.is_downloading_completed {
                let texture = gdk::Texture::from_filename(&file.local.path).unwrap();
                self.imp().avatar.set_custom_image(Some(&texture));
            } else {
                let file_id = file.id;

                spawn(clone!(@weak self as obj, @weak session => async move {
                    obj.download_avatar(file_id, &session).await;
                }));
            }
        } else {
            self.imp().avatar.set_custom_image(gdk::Paintable::NONE);
        }
    }

    fn update_avatar(&self) {
        let imp = self.imp();

        imp.avatar.set_custom_image(gdk::Paintable::NONE);
        imp.avatar.set_icon_name(None);
        imp.avatar.set_show_initials(true);

        if let Some(item) = self.item() {
            if let Some(user) = item.downcast_ref::<User>() {
                if user.type_().0 == UserType::Deleted {
                    imp.avatar.set_icon_name(Some("ghost-symbolic"));
                    imp.avatar.set_show_initials(false);
                } else {
                    self.load_image(user.avatar(), user.session());
                }
            } else if let Some(chat) = item.downcast_ref::<Chat>() {
                if chat.is_own_chat() {
                    imp.avatar.set_icon_name(Some("user-bookmarks-symbolic"));
                    imp.avatar.set_show_initials(false);
                } else {
                    self.load_image(chat.avatar(), chat.session());
                }
            }
        }
    }

    fn update_display_name(&self) {
        let imp = self.imp();

        if let Some(item) = self.item() {
            if let Some(user) = item.downcast_ref::<User>() {
                imp.avatar
                    .set_text(Some(&strings::user_display_name(user, true)));
            } else if let Some(chat) = item.downcast_ref::<Chat>() {
                if chat.is_own_chat() {
                    imp.avatar.set_text(Some("-"));
                } else {
                    imp.avatar.set_text(Some(chat.title().as_ref()));
                };
            }
        }
    }

    async fn download_avatar(&self, file_id: i32, session: &Session) {
        match session.download_file(file_id).await {
            Ok(file) => {
                let texture = gdk::Texture::from_filename(file.local.path).unwrap();
                self.imp().avatar.set_custom_image(Some(&texture));
            }
            Err(e) => {
                log::warn!("Failed to download an avatar: {e:?}");
            }
        }
    }

    pub(crate) fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().clone()
    }

    pub(crate) fn set_item(&self, item: Option<glib::Object>) {
        let imp = self.imp();

        imp.chat_signal_group
            .get()
            .unwrap()
            .set_target(item.as_ref().and_then(|item| item.downcast_ref::<Chat>()));
        imp.user_signal_group
            .get()
            .unwrap()
            .set_target(item.as_ref().and_then(|item| item.downcast_ref::<User>()));

        imp.item.replace(item);

        self.update_display_name();
        self.update_avatar();

        self.notify("item");
    }

    pub(crate) fn custom_text(&self) -> Option<String> {
        self.imp().avatar.text().map(Into::into)
    }

    pub(crate) fn set_custom_text(&self, text: Option<&str>) {
        self.imp().avatar.set_text(text);
        self.notify("custom-text");
    }

    pub(crate) fn size(&self) -> i32 {
        self.imp().avatar.size()
    }

    pub(crate) fn set_size(&self, size: i32) {
        self.imp().avatar.set_size(size);
        self.notify("size");
    }
}
