use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Bar)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/chat_folder/bar.ui")]
    pub(crate) struct Bar {
        #[property(get, set)]
        pub(super) chat_folder_list: glib::WeakRef<model::ChatFolderList>,
        #[property(get, set)]
        pub(super) selected_chat_list: glib::WeakRef<model::ChatList>,
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub(super) selection: TemplateChild<ui::SidebarChatFolderSelection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Bar {
        const NAME: &'static str = "PaplSidebarChatFolderBar";
        type Type = super::Bar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            klass.set_css_name("chatfolderbar");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Bar {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn dispose(&self) {
            utils::unparent_children(&*self.obj());
        }
    }

    impl WidgetImpl for Bar {}

    #[gtk::template_callbacks]
    impl Bar {
        #[template_callback]
        fn on_notify_chat_folder_list(&self) {
            if self.obj().chat_folder_list().is_some() {
                self.on_list_view_activated(0);
            }
        }

        #[template_callback]
        fn on_scroll_vertical(
            &self,
            _dx: f64,
            dy: f64,
            _scroll: gtk::EventControllerScroll,
        ) -> glib::Propagation {
            let adj = self.scrolled_window.hadjustment();
            adj.set_value(adj.value() + dy * 25.0);

            glib::Propagation::Proceed
        }

        #[template_callback]
        fn on_list_view_activated(&self, position: u32) {
            self.selection.select_item(position, true);
        }

        #[template_callback]
        fn on_signal_list_item_factory_bind(&self, list_item: &glib::Object) {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            list_item.set_selectable(false);

            let chat_list = list_item.item().and_downcast::<model::ChatList>().unwrap();

            list_item.set_child(Some(
                &adw::Bin::builder()
                    .child(&ui::SidebarChatFolderRow::from(&chat_list))
                    .margin_top(6)
                    .margin_bottom(6)
                    .build(),
            ));
        }

        #[template_callback]
        fn on_signal_list_item_factory_unbind(&self, list_item: &glib::Object) {
            list_item
                .downcast_ref::<gtk::ListItem>()
                .unwrap()
                .set_child(gtk::Widget::NONE);
        }
    }
}

glib::wrapper! {
    pub(crate) struct Bar(ObjectSubclass<imp::Bar>)
        @extends gtk::Widget;
}
