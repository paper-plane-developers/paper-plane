use std::cell::Cell;
use std::sync::OnceLock;

use glib::clone;
use glib::Properties;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::strings;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::ArchiveRow)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/archive_row.ui")]
    pub(crate) struct ArchiveRow {
        pub(super) settings: utils::PaperPlaneSettings,
        #[property(get, set = Self::set_archive_chat_list)]
        pub(super) archive_chat_list: glib::WeakRef<model::ChatList>,
        #[property(get, set)]
        pub(super) collapsed: Cell<bool>,
        #[property(get, set)]
        pub(super) in_main_menu: Cell<bool>,
        #[template_child]
        pub(super) popover_menu: TemplateChild<gtk::PopoverMenu>,
        #[template_child]
        pub(super) button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) chats_inscription: TemplateChild<gtk::Inscription>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArchiveRow {
        const NAME: &'static str = "PaplSidebarArchiveRow";
        type Type = super::ArchiveRow;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Actionable,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.set_css_name("archiverow");

            klass.install_action("archive-row.collapse", None, |widget, _, _| {
                widget.collapse();
            });
            klass.install_action("archive-row.expand", None, |widget, _, _| {
                widget.expand();
            });

            klass.install_action("archive-row.move-to-main-menu", None, |widget, _, _| {
                widget.move_to_main_menu();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArchiveRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                Self::derived_properties()
                    .iter()
                    .cloned()
                    .chain(vec![
                        glib::ParamSpecString::builder("action-name")
                            .explicit_notify()
                            .build(),
                        glib::ParamSpecVariant::builder("action-target", glib::VariantTy::ANY)
                            .explicit_notify()
                            .build(),
                    ])
                    .collect::<Vec<_>>()
            })
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "action-name" => self.set_action_name(value.get().unwrap()),
                "action-target" => self.set_action_target_value(
                    value.get::<Option<glib::Variant>>().unwrap().as_ref(),
                ),
                _ => self.derived_set_property(id, value, pspec),
            }
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "action-name" => self.action_name().to_value(),
                "action-target" => self.action_name().to_value(),
                _ => self.derived_property(id, pspec),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = &*self.obj();

            self.popover_menu.set_parent(obj);

            self.settings
                .bind("collapsed-archive-row", obj, "collapsed")
                .build();

            self.settings
                .bind("archive-row-in-main-menu", obj, "in-main-menu")
                .build();

            self.on_notify_collapsed();
        }

        fn dispose(&self) {
            utils::unparent_children(&*self.obj());
        }
    }

    impl WidgetImpl for ArchiveRow {}

    impl ActionableImpl for ArchiveRow {
        fn action_name(&self) -> Option<glib::GString> {
            self.button.action_name()
        }

        fn action_target_value(&self) -> Option<glib::Variant> {
            self.button.action_target_value()
        }

        fn set_action_name(&self, name: Option<&str>) {
            self.button.set_action_name(name);
        }

        fn set_action_target_value(&self, value: Option<&glib::Variant>) {
            self.button.set_action_target_value(value);
        }
    }

    #[gtk::template_callbacks]
    impl ArchiveRow {
        fn set_archive_chat_list(&self, archive_chat_list: &model::ChatList) {
            let obj = &*self.obj();
            if Some(archive_chat_list) == obj.archive_chat_list().as_ref() {
                return;
            }

            archive_chat_list.connect_items_changed(
                clone!(@weak obj => move |chat_list, _, _, _| {
                    obj.set_chats_label(chat_list);
                }),
            );
            obj.set_chats_label(archive_chat_list);

            self.archive_chat_list.set(Some(archive_chat_list));
            obj.notify_archive_chat_list();
        }

        #[template_callback]
        fn on_notify_collapsed(&self) {
            let obj = &*self.obj();
            let collaped = obj.collapsed();

            if collaped {
                obj.remove_css_class("expanded");
                obj.add_css_class("collapsed");
            } else {
                obj.remove_css_class("collapsed");
                obj.add_css_class("expanded");
            }

            obj.action_set_enabled("archive-row.collapse", !collaped);
            obj.action_set_enabled("archive-row.expand", collaped);
        }

        #[template_callback]
        fn on_notify_in_main_menu(&self) {
            let obj = &*self.obj();
            let in_main_menu = obj.in_main_menu();

            if in_main_menu {
                obj.add_css_class("in-main-menu");
            } else {
                obj.remove_css_class("in-main-menu");
            }

            obj.action_set_enabled("archive-row.move-to-main-menu", !in_main_menu);
        }

        #[template_callback]
        fn on_right_click(&self, _: i32, x: f64, y: f64) {
            self.popover_menu
                .set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 0, 0)));
            self.popover_menu
                .set_halign(if self.obj().direction() == gtk::TextDirection::Rtl {
                    gtk::Align::End
                } else {
                    gtk::Align::Start
                });
            self.popover_menu.popup();
        }
    }
}

glib::wrapper! {
    pub(crate) struct ArchiveRow(ObjectSubclass<imp::ArchiveRow>)
        @extends gtk::Widget,
        @implements gtk::Actionable;
}

impl ArchiveRow {
    pub(crate) fn collapse(&self) {
        self.set_collapsed(true);
    }

    pub(crate) fn expand(&self) {
        self.set_collapsed(false);
    }

    pub(crate) fn move_to_main_menu(&self) {
        self.imp()
            .settings
            .set("archive-row-in-main-menu", true)
            .unwrap();
    }

    fn set_chats_label(&self, chat_list: &model::ChatList) {
        let text = chat_list
            .iter()
            .map(Result::unwrap)
            .map(|item: glib::Object| {
                strings::chat_display_name(
                    &item
                        .downcast::<model::ChatListItem>()
                        .unwrap()
                        .chat()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        self.imp().chats_inscription.set_text(Some(&text));
    }
}
