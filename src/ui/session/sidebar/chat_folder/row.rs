use std::cell::OnceCell;

use gettextrs::gettext;
use glib::closure;
use glib::Properties;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::i18n::gettext_f;
use crate::model;
use crate::ui;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Row)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/chat_folder/row.ui")]
    pub(crate) struct Row {
        #[property(get, set, construct_only)]
        pub(super) chat_folder_bar: OnceCell<ui::SidebarChatFolderBar>,
        #[property(get, set)]
        pub(super) chat_list: glib::WeakRef<model::ChatList>,
        #[template_child]
        pub(super) title_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "PaplSidebarChatFolderRow";
        type Type = super::Row;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            klass.set_css_name("chatfolderrow");

            klass.install_action_async("chat-folder-row.remove", None, |widget, _, _| async move {
                widget.remove_chat_folder().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
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

            gtk::ClosureExpression::new::<String>(
                [
                    &chat_list_expr.chain_property::<model::ChatList>("list-type"),
                    &chat_list_expr.chain_property::<model::ChatList>("title"),
                ],
                closure!(
                    |_: Self::Type, list_type: model::BoxedChatListType, title: String| {
                        use tdlib::enums::ChatList;

                        match list_type.0 {
                            ChatList::Main => gettext("All Chats"),
                            ChatList::Archive => gettext("Archived Chats"),
                            _ => title,
                        }
                    }
                ),
            )
            .bind(&self.title_label.get(), "label", Some(obj));
        }

        fn dispose(&self) {
            utils::unparent_children(&*self.obj());
        }
    }

    impl WidgetImpl for Row {}

    #[gtk::template_callbacks]
    impl Row {
        #[template_callback]
        fn on_notify_chat_list(&self) {
            let obj = self.obj();
            obj.action_set_enabled("chat-folder-row.remove", obj.is_folder());
        }

        #[template_callback]
        fn on_button_3_pressed(gesture: &gtk::GestureClick, _: i32, _: f64, _: f64) {
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }

        #[template_callback]
        fn on_button_3_released(&self, _n_press: i32, x: f64, y: f64) {
            self.show_menu(x as i32, y as i32);
        }

        #[template_callback]
        fn on_long_pressed(&self, x: f64, y: f64) {
            self.show_menu(x as i32, y as i32);
        }

        fn show_menu(&self, x: i32, y: i32) {
            let obj = &*self.obj();

            if obj.is_folder() {
                let popover_menu = obj.chat_folder_bar().popover_menu();

                popover_menu.unparent();
                popover_menu.set_parent(obj);
                popover_menu.set_pointing_to(Some(&gdk::Rectangle::new(x, y, 0, 0)));
                popover_menu.set_halign(match obj.direction() {
                    gtk::TextDirection::Ltr => gtk::Align::Start,
                    _ => gtk::Align::End,
                });

                popover_menu.popup();
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}

impl Row {
    pub(crate) fn new(
        chat_folder_bar: &ui::SidebarChatFolderBar,
        chat_list: &model::ChatList,
    ) -> Self {
        glib::Object::builder()
            .property("chat-folder-bar", chat_folder_bar)
            .property("chat-list", chat_list)
            .build()
    }

    pub(crate) async fn remove_chat_folder(&self) {
        if let Some(chat_list) = self.chat_list() {
            if let Err(e) = chat_list.delete().await {
                utils::show_toast(
                    self,
                    gettext_f(
                        "Failed to remove folder: {error}",
                        &[("error", &e.to_string())],
                    ),
                );
            }
        }
    }

    pub(crate) fn is_folder(&self) -> bool {
        self.chat_list()
            .filter(|chat_list| {
                matches!(
                    chat_list.list_type().0,
                    tdlib::enums::ChatList::Folder { .. }
                )
            })
            .is_some()
    }
}
