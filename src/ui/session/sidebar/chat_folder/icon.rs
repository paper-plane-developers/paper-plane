use std::cell::Cell;

use glib::closure;
use glib::Properties;
use gtk::gdk;
use gtk::glib;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Icon)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/chat_folder/icon.ui")]
    pub(crate) struct Icon {
        pub(super) unread_label_width: Cell<i32>,
        pub(super) unread_label_height: Cell<i32>,
        #[property(get, set)]
        pub(super) chat_list: glib::WeakRef<model::ChatList>,
        #[template_child]
        pub(super) icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) unread_mask_bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) unread_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Icon {
        const NAME: &'static str = "PaplSidebarChatFolderIcon";
        type Type = super::Icon;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("chatfoldericon");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Icon {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = &*self.obj();

            let chat_list_expr = Self::Type::this_expression("chat-list");
            let unread_count_expr =
                chat_list_expr.chain_property::<model::ChatList>("unread-chat-count");

            gtk::ClosureExpression::new::<String>(
                [
                    &chat_list_expr.chain_property::<model::ChatList>("list-type"),
                    &chat_list_expr.chain_property::<model::ChatList>("icon"),
                ],
                closure!(
                    |_: Self::Type, list_type: model::BoxedChatListType, icon: &str| {
                        use tdlib::enums::ChatList;

                        match list_type.0 {
                            ChatList::Main => "all-chats-symbolic",
                            ChatList::Archive => "",
                            _ => match icon {
                                "Airplane" => "airplane-mode-symbolic",
                                "All" => "all-chats-symbolic",
                                // "Book" => "",
                                // "Bots" => "",
                                // "Cat" => "",
                                // "Channels" => "",
                                // "Crown" => "",
                                "Custom" => "folder-symbolic",
                                // "Favorite" => "",
                                // "Flower" => "",
                                "Game" => "applications-games-symbolic",
                                // "Groups" => "",
                                "Home" => "user-home-symbolic",
                                // "Light" => "",
                                // "Like" => "",
                                "Love" => "emote-love-symbolic",
                                // "Mask" => "",
                                // "Money" => "",
                                // "Note" => "",
                                // "Palette" => "",
                                // "Party" => "",
                                // "Private" => "",
                                // "Setup" => "",
                                // "Sport" => "",
                                // "Study" => "",
                                // "Trade" => "",
                                "Travel" => "emoji-travel-symbolic",
                                // "Unmuted" => "",
                                "Unread" => "mail-unread-symbolic",
                                // "Work" => "",
                                _ => "folder-symbolic",
                            },
                        }
                    }
                ),
            )
            .bind(&self.icon.get(), "icon-name", Some(obj));

            unread_count_expr.bind(&self.unread_label.get(), "visible", Some(obj));
            unread_count_expr.bind(&self.unread_label.get(), "label", Some(obj));
        }

        fn dispose(&self) {
            utils::unparent_children(&*self.obj());
        }
    }

    impl WidgetImpl for Icon {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let (min1, nat1, min_baseline1, nat_baseline1) =
                self.icon.measure(orientation, for_size);
            let (min2, nat2, min_baseline2, nat_baseline2) =
                self.unread_mask_bin.measure(orientation, for_size);
            let (min3, nat3, min_baseline3, nat_baseline3) =
                self.unread_label.measure(orientation, for_size);

            match orientation {
                gtk::Orientation::Horizontal => self.unread_label_width.set(min3),
                _ => self.unread_label_height.set(min3),
            }

            (
                min1.max(min2).max(min3),
                nat1.max(nat2).max(nat3),
                min_baseline1.max(min_baseline2).max(min_baseline3),
                nat_baseline1.max(nat_baseline2).max(nat_baseline3),
            )
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let unread_label_width = self.unread_label_width.get();
            let unread_label_height = self.unread_label_height.get();

            self.icon
                .size_allocate(&gdk::Rectangle::new(0, 0, width, height), baseline);

            self.unread_mask_bin.size_allocate(
                &gdk::Rectangle::new(
                    match self.obj().direction() {
                        gtk::TextDirection::Ltr => width - unread_label_width - 2,
                        _ => 0,
                    },
                    0,
                    unread_label_width + 2,
                    unread_label_height + 2,
                ),
                baseline,
            );

            self.unread_label
                .size_allocate(&gdk::Rectangle::new(0, 0, width, height), baseline);
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = &*self.obj();

            if obj
                .chat_list()
                .filter(|chat_list| chat_list.unread_chat_count() > 0)
                .is_none()
            {
                obj.snapshot_child(&*self.icon, snapshot);
                return;
            }

            snapshot.push_mask(gsk::MaskMode::InvertedAlpha);

            obj.snapshot_child(&*self.unread_mask_bin, snapshot);
            snapshot.pop();

            obj.snapshot_child(&*self.icon, snapshot);
            snapshot.pop();

            obj.snapshot_child(&*self.unread_label, snapshot);
        }
    }
}

glib::wrapper! {
    pub(crate) struct Icon(ObjectSubclass<imp::Icon>)
        @extends gtk::Widget;
}
