use glib::Binding;
use glib::BindingFlags;
use glib::ParamFlags;
use glib::ParamSpec;
use glib::Value;
use gtk::glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use once_cell::sync::Lazy;
use std::rc::Rc;
use tdgrand::enums::ProxyType;
use tdgrand::functions;
use tdgrand::types;
use tdgrand::types::Proxy;

use crate::proxy::proxy_window::ProxyWindow;
use crate::utils::do_async;
mod imp {
    use super::*;
    use glib::WeakRef;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/proxy-row.ui")]
    pub struct ProxyRow {
        pub proxy: Rc<RefCell<Proxy>>,
        pub bindings: RefCell<Vec<Binding>>,
        pub proxy_window: WeakRef<ProxyWindow>,
        #[template_child]
        pub check_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub type_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub server_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub state_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyRow {
        const NAME: &'static str = "ProxyRow";
        type Type = super::ProxyRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("proxy.delete", None, move |widget, _, _| {
                widget.delete_proxy();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProxyRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpec::new_boolean(
                        "enabled",
                        "enabled",
                        "enabled",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpec::new_string("type", "type", "type", None, ParamFlags::READWRITE),
                    ParamSpec::new_string(
                        "server",
                        "server",
                        "server",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpec::new_object(
                        "proxy-window",
                        "proxy-window",
                        "The proxy main window",
                        ProxyWindow::static_type(),
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "proxy-window" => self.proxy_window.set(Some(&value.get().unwrap())),
                "enabled" => {
                    let input_value = value.get().expect("The value needs to be of type `bool`.");
                    self.proxy.borrow_mut().is_enabled = input_value;
                }
                "type" => {}
                "server" => {}
                _ => unimplemented!(),
            }
        }
        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "proxy-window" => obj.window().to_value(),
                "enabled" => self.proxy.borrow().is_enabled.to_value(),
                "type" => match self.proxy.borrow().r#type {
                    ProxyType::Http(_) => "HTTP  ".to_value(),
                    ProxyType::Socks5(_) => "SOCKS5".to_value(),
                    _ => unimplemented!(),
                },
                "server" => format!(
                    "{}:{}",
                    self.proxy.borrow().server,
                    self.proxy.borrow().port
                )
                .to_value(),
                _ => unimplemented!(),
            }
        }
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_bindings();
        }
    }
    impl BoxImpl for ProxyRow {}
    impl ListBoxRowImpl for ProxyRow {}
    impl WidgetImpl for ProxyRow {}
}

glib::wrapper! {
    pub struct ProxyRow(ObjectSubclass<imp::ProxyRow>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ProxyRow {
    pub fn new(proxy_window: &ProxyWindow) -> Self {
        glib::Object::new(&[("proxy-window", proxy_window)]).expect("Fail to create ProxyRow")
    }

    pub fn check_button_set_group(&self, button: Option<&gtk::CheckButton>) {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.check_button.set_group(button);
    }

    pub fn check_button(&self) -> gtk::CheckButton {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.check_button.get()
    }

    fn handle_proxy_test_result<T>(&self, result: Result<T, types::Error>) {
        let self_ = imp::ProxyRow::from_instance(self);
        match result {
            Err(_err) => {
                self_.state_label.set_text("not available");
                self_.state_label.set_css_classes(&["proxy-state-text-red"]);
            }
            Ok(_) => {
                self_.state_label.set_text("available");
                self_
                    .state_label
                    .set_css_classes(&["proxy-state-text-green"]);
            }
        }
    }

    fn enable_proxy(&self) {
        let self_ = imp::ProxyRow::from_instance(self);
        let proxy_id = self_.proxy.borrow().id;
        let client_id = self.client_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::EnableProxy::new()
                    .proxy_id(proxy_id)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as app => move |result| async move {
                match result {
                    Err(err) => {
                        log::error!("Received an error for EnableProxy: {}", err.code);
                    },
                    Ok(_) => {},
                }
            }),
        );
    }

    fn test_proxy(&self) {
        let self_ = imp::ProxyRow::from_instance(self);

        let server = self_.proxy.borrow().server.clone();
        let port = self_.proxy.borrow().port;
        let type_ = self_.proxy.borrow().r#type.clone();
        let client_id = self.client_id();
        let timeout = 5.0;

        self_.state_label.set_text("connecting");
        self_
            .state_label
            .set_css_classes(&["proxy-state-text-cyan"]);
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::TestProxy::new()
                    .server(server)
                    .dc_id(1)
                    .port(port)
                    .r#type(type_)
                    .timeout(timeout)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as app => move |result| async move {
                app.handle_proxy_test_result(result);

            }),
        );
    }

    fn delete_proxy(&self) {
        let self_ = imp::ProxyRow::from_instance(self);
        let client_id = self.client_id();
        let proxy_id = self_.proxy.borrow().id;

        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::RemoveProxy::new()
                    .proxy_id(proxy_id)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                match result  {
                    Err(_err) => {}
                    Ok(_) => {
                        obj.unparent()
                    },
                }
            }),
        );
    }

    fn client_id(&self) -> i32 {
        self.window().client_id()
    }

    fn window(&self) -> ProxyWindow {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.proxy_window.upgrade().unwrap()
    }

    fn set_proxy(&self, proxy: Proxy) {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.proxy.replace(proxy);
    }

    pub fn bind_proxy(&self, proxy: Proxy) {
        let self_ = imp::ProxyRow::from_instance(self);
        self.set_proxy(proxy);

        let mut bindings = self_.bindings.borrow_mut();
        let checkt_button = self_.check_button.get();
        let server_label = self_.server_label.get();
        let type_label = self_.type_label.get();

        let check_button_bind = self
            .bind_property("enabled", &checkt_button, "active")
            .flags(BindingFlags::SYNC_CREATE | BindingFlags::BIDIRECTIONAL)
            .build()
            .expect("Could not bind properties");
        bindings.push(check_button_bind);
        let type_label_bind = self
            .bind_property("type", &type_label, "label")
            .flags(BindingFlags::SYNC_CREATE)
            .build()
            .expect("Could not bind properties");
        bindings.push(type_label_bind);
        let server_label_bind = self
            .bind_property("server", &server_label, "label")
            .flags(BindingFlags::SYNC_CREATE)
            .build()
            .expect("Could not bind properties");
        bindings.push(server_label_bind);
    }

    fn setup_bindings(&self) {
        let self_ = imp::ProxyRow::from_instance(self);
        self_
            .check_button
            .connect_active_notify(clone!(@weak self as app => move |check_button| {
                if check_button.is_active() {
                    app.show_state_label();
                    app.test_proxy();
                    app.enable_proxy();
                } else {
                    app.hide_state_label();
                }
            }));
    }

    pub fn show_state_label(&self) {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.state_label.set_visible(true);
    }
    pub fn hide_state_label(&self) {
        let self_ = imp::ProxyRow::from_instance(self);
        self_.state_label.set_visible(false);
    }

    pub fn unbind(&self) {
        let self_ = imp::ProxyRow::from_instance(self);
        for binding in self_.bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}
