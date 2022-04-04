use glib::{clone, closure};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use tdlib::types::File;

use crate::session::{Avatar as AvatarItem, Chat, User};
use crate::{expressions, Session};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/components-avatar.ui")]
    pub(crate) struct Avatar {
        /// A `Chat` or `User`
        pub(super) item: RefCell<Option<glib::Object>>,
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
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
                        "The item of the avatar",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "custom-text",
                        "Custom Text",
                        "The custom text of the avatar",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecInt::new(
                        "size",
                        "Size",
                        "The size of the avatar",
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
                "custom-text" => obj.set_custom_text(value.get().unwrap()),
                "size" => obj.set_size(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                "custom-text" => obj.custom_text().to_value(),
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
        let imp = self.imp();

        // Chat title expression
        let title_expression = item_expression.chain_property::<Chat>("title");
        gtk::ClosureExpression::new::<String, _, _>(
            &[item_expression.clone(), title_expression],
            closure!(|_: Self, item: Option<glib::Object>, title: String| {
                item.as_ref()
                    .and_then(|i| i.downcast_ref::<Chat>())
                    .filter(|chat| chat.is_own_chat())
                    // Workaround for having a blue AdwAvatar for Saved Messages chat
                    .map(|_| "-".to_string())
                    .unwrap_or(title)
            }),
        )
        .bind(&*imp.avatar, "text", Some(self));

        // User title expression
        expressions::user_full_name(&item_expression).bind(&*imp.avatar, "text", Some(self));

        // Icon expression
        let icon_name_expression =
            item_expression.chain_closure::<Option<String>>(closure!(|_: Self,
                                                                      item: Option<
                glib::Object,
            >| {
                item.as_ref()
                    .and_then(|i| i.downcast_ref::<Chat>())
                    .filter(|chat| chat.is_own_chat())
                    // Show bookmark icon for Saved Messages chat
                    .map(|_| "user-bookmarks-symbolic")
            }));
        icon_name_expression.bind(&*imp.avatar, "icon-name", Some(self));

        // Don't show the initials if we show a custom icon
        icon_name_expression
            .chain_closure::<bool>(closure!(
                |_: Self, icon_name: Option<String>| icon_name.is_none()
            ))
            .bind(&*imp.avatar, "show-initials", Some(self));
    }

    fn load_image(&self, avatar_item: Option<AvatarItem>, session: Session) {
        if let Some(avatar_item) = avatar_item {
            let file = avatar_item.0;
            if file.local.is_downloading_completed {
                let texture = gdk::Texture::from_filename(&file.local.path).unwrap();
                self.imp().avatar.set_custom_image(Some(&texture));
            } else {
                let (sender, receiver) =
                    glib::MainContext::sync_channel::<File>(Default::default(), 5);

                receiver.attach(
                    None,
                    clone!(@weak self as obj => @default-return glib::Continue(false), move |file| {
                        if file.local.is_downloading_completed {
                            let texture = gdk::Texture::from_filename(&file.local.path).unwrap();
                            obj.imp().avatar.set_custom_image(Some(&texture));
                        }
                        glib::Continue(true)
                    }),
                );

                session.download_file(file.id, sender);
            }
        } else {
            self.imp().avatar.set_custom_image(gdk::Paintable::NONE);
        }
    }

    pub(crate) fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().clone()
    }

    pub(crate) fn set_item(&self, item: Option<glib::Object>) {
        let imp = self.imp();

        if let Some(id) = imp.handler_id.take() {
            self.item().unwrap().disconnect(id);
        }

        if let Some(ref item) = item {
            if let Some(chat) = item.downcast_ref::<Chat>() {
                if chat.is_own_chat() {
                    imp.avatar.set_custom_image(gdk::Paintable::NONE);
                } else {
                    self.load_image(chat.avatar(), chat.session());
                    let handler_id =
                        chat.connect_avatar_notify(clone!(@weak self as obj => move |chat, _| {
                            obj.load_image(chat.avatar(), chat.session());
                        }));
                    imp.handler_id.replace(Some(handler_id));
                }
            } else if let Some(user) = item.downcast_ref::<User>() {
                self.load_image(user.avatar(), user.session());
                let handler_id =
                    user.connect_avatar_notify(clone!(@weak self as obj => move |user, _| {
                        obj.load_image(user.avatar(), user.session());
                    }));
                imp.handler_id.replace(Some(handler_id));
            } else {
                imp.avatar.set_custom_image(gdk::Paintable::NONE);
            }
        } else {
            imp.avatar.set_custom_image(gdk::Paintable::NONE);
        }

        imp.item.replace(item);
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
