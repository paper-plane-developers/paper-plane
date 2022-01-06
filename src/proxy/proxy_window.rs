use super::proxy_handle_dialog::ProxyHandleDialog;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums, functions};

use crate::config;
use crate::proxy::proxy_row::ProxyRow;
use crate::utils::do_async;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/proxy-window.ui")]
    pub struct ProxyWindow {
        pub client_id: Cell<i32>,
        #[template_child]
        pub proxy_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub proxy_handle_dialog: TemplateChild<ProxyHandleDialog>,
        #[template_child]
        pub proxy_enable_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub proxy_add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub proxy_list: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyWindow {
        const NAME: &'static str = "ProxyWindow";
        type Type = super::ProxyWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProxyWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_int(
                    "client-id",
                    "Client Id",
                    "The telegram client id",
                    std::i32::MIN,
                    std::i32::MAX,
                    0,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "client-id" => {
                    let client_id = value.get().unwrap();
                    self.client_id.set(client_id);
                }
                _ => unimplemented!(),
            }
        }
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let self_ = imp::ProxyWindow::from_instance(obj);

            let use_proxy_switch = &*self_.proxy_enable_switch;
            let settings = gio::Settings::new(config::APP_ID);
            settings
                .bind("use-proxy", use_proxy_switch, "state")
                .build();

            if self_.proxy_enable_switch.is_active() {
                obj.proxy_set_enable()
            } else {
                obj.proxy_set_disable()
            }

            obj.setup_bindings();
            obj.init_exits_proxies();
        }
    }
    impl WidgetImpl for ProxyWindow {}
    impl WindowImpl for ProxyWindow {}
    impl AdwWindowImpl for ProxyWindow {}
    impl PreferencesWindowImpl for ProxyWindow {}
}

glib::wrapper! {
    pub struct ProxyWindow(ObjectSubclass<imp::ProxyWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;

}

impl ProxyWindow {
    pub fn new(client_id: i32) -> Self {
        glib::Object::new(&[("client-id", &client_id)]).expect("Failed to create ProxyWindow")
    }

    fn setup_bindings(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);

        self_.proxy_enable_switch.connect_active_notify(
            clone!(@weak self as app => move |switch| {
                if switch.is_active() {
                     app.proxy_set_enable()
                } else {
                    app.proxy_set_disable()
                }
            }),
        );

        self_
            .proxy_add_button
            .connect_clicked(clone!(@weak self as app => move |_| {
                app.show_proxy_handle_dialog();
            }));

        self_
            .proxy_stack
            .connect_visible_child_notify(clone!(@weak self as  app => move |_| {
                app.update_actions_for_visible_page()
            }));
    }

    fn update_actions_for_visible_page(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);

        let visible_page = self_.proxy_stack.visible_child_name().unwrap();
        match visible_page.as_str() {
            "main-page" => {
                self.init_exits_proxies();
                self.set_default_height(300);
            }
            "proxy-handle" => {
                self.set_default_height(300);
            }
            _ => {}
        };
    }

    fn init_exits_proxies(&self) {
        let client_id = self.client_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move { functions::GetProxies::new().send(client_id).await },
            clone!(@weak self as obj => move |result| async move {
                match result {
                    Err(err) => {
                        log::error!("GetProxies Error: {}", err.message)
                    }
                    Ok(proxies) => {
                        obj.set_proxies(&proxies);
                    },
                }
            }),
        );
    }
    fn proxy_set_enable(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.proxy_list.get().set_sensitive(true);
        self_.proxy_add_button.get().set_sensitive(true);
        self.init_exits_proxies();
    }

    fn proxy_set_disable(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.proxy_list.get().set_sensitive(false);
        self_.proxy_add_button.get().set_sensitive(false);

        let list_model = self_.proxy_list.get().observe_children();

        for i in 0..list_model.n_items() {
            if let Some(item) = list_model.item(i) {
                item.dynamic_cast::<ProxyRow>().unwrap().disable_proxy();
            };
        }
    }

    fn set_proxies(&self, proxies: &enums::Proxies) {
        let self_ = imp::ProxyWindow::from_instance(self);

        match proxies {
            enums::Proxies::Proxies(proxies) => {
                let mut last_check_button: Option<gtk::CheckButton> = None;
                while let Some(child) = self_.proxy_list.last_child() {
                    self_.proxy_list.remove(&child);
                }
                for proxy in proxies.proxies.clone().into_iter() {
                    let proxy_row = ProxyRow::new(self);
                    proxy_row.bind_proxy(proxy);
                    if let Some(button) = &last_check_button {
                        proxy_row.check_button_set_group(Some(&button))
                    }
                    last_check_button = Some(proxy_row.check_button());
                    self_.proxy_list.append(&proxy_row);
                }
            }
        };
    }

    fn show_proxy_handle_dialog(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.proxy_handle_dialog.set_client_id(self.client_id());
        self_.proxy_stack.set_visible_child_name("proxy-handle");
    }

    pub fn client_id(&self) -> i32 {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.client_id.get()
    }

    pub fn cast_to_main_stack(&self) {
        let self_ = imp::ProxyWindow::from_instance(self);
        self_.proxy_stack.set_visible_child_name("main-page");
    }
}
