use crate::utils::do_async;
use adw::prelude::*;
use glib::clone;
use gtk::{glib, subclass::prelude::*, CompositeTemplate};
use tdgrand::{enums, functions, types};

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::GEnum)]
#[repr(u32)]
#[genum(type_name = "ProxyTypes")]
pub enum ProxyTypes {
    #[genum(name = "Socks5", nick = "socks5")]
    Socks5,
    #[genum(name = "Http", nick = "http")]
    Http,
}

impl Default for ProxyTypes {
    fn default() -> Self {
        Self::Http
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::Cell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/proxy-handle-dialog.ui")]
    pub struct ProxyHandleDialog {
        pub client_id: Cell<i32>,
        #[template_child]
        pub proxy_types: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub proxy_address_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_port_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_auth_username_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_auth_passwd_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub proxy_save_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyHandleDialog {
        const NAME: &'static str = "ProxyHandleDialog";
        type Type = super::ProxyHandleDialog;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("proxy.save-proxy", None, move |widget, _, _| {
                widget.add_save_proxy();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProxyHandleDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_bindings();
        }
    }

    impl WidgetImpl for ProxyHandleDialog {}
    impl WindowImpl for ProxyHandleDialog {}
    impl AdwWindowImpl for ProxyHandleDialog {}
}

glib::wrapper! {
    pub struct ProxyHandleDialog(ObjectSubclass<imp::ProxyHandleDialog>)
        @extends gtk::Widget;
}

fn is_non_ascii_digit(c: char) -> bool {
    !c.is_ascii_digit()
}

impl ProxyHandleDialog {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ProxyHandleDialog")
    }

    pub fn set_client_id(&self, client_id: i32) {
        let self_ = imp::ProxyHandleDialog::from_instance(self);
        self_.client_id.set(client_id);
    }

    fn setup_bindings(&self) {
        let self_ = imp::ProxyHandleDialog::from_instance(self);

        // port validator
        self_
            .proxy_port_entry
            .connect_text_notify(clone!(@weak self as app => move |widget| {
                let text = widget.text();
                if text.contains(is_non_ascii_digit) {
                    widget.set_text(&text.replace(is_non_ascii_digit, ""))
                }
            }));
    }

    fn proxy_type(&self) -> tdgrand::enums::ProxyType {
        let self_ = imp::ProxyHandleDialog::from_instance(self);
        let passwd = self_.proxy_auth_passwd_entry.text().to_string();
        let username = self_.proxy_auth_username_entry.text().to_string();
        if let Some(selected_item) = self_.proxy_types.selected_item() {
            return match selected_item
                .downcast::<adw::EnumListItem>()
                .unwrap()
                .nick()
                .unwrap()
                .as_str()
            {
                "socks5" => {
                    let mut proxy = tdgrand::types::ProxyTypeSocks5::default();
                    proxy.username = username;
                    proxy.password = passwd;
                    enums::ProxyType::Socks5(proxy)
                }
                "http" => {
                    let mut proxy = tdgrand::types::ProxyTypeHttp::default();
                    proxy.username = username;
                    proxy.password = passwd;
                    enums::ProxyType::Http(proxy)
                }
                _ => enums::ProxyType::Socks5(types::ProxyTypeSocks5::default()),
            };
        };
        enums::ProxyType::Socks5(Default::default())
    }

    fn client_id(&self) -> i32 {
        let self_ = imp::ProxyHandleDialog::from_instance(self);
        self_.client_id.get()
    }

    fn add_save_proxy(&self) {
        let self_ = imp::ProxyHandleDialog::from_instance(self);
        let address = self_.proxy_address_entry.text().to_string();
        let port = self_
            .proxy_port_entry
            .text()
            .to_string()
            .parse::<i32>()
            .unwrap_or(8080);
        let client_id = self.client_id();
        let proxy_type = self.proxy_type();

        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::AddProxy::new()
                    .port(port)
                    .server(address)
                    .r#type(proxy_type)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as app => move |result| async move {
                app.handle_proxy_result(result);
            }),
        );
        let p = self.parent().unwrap();
        // back to main page
        p.dynamic_cast::<gtk::Stack>()
            .unwrap()
            .set_visible_child_name("main-page");
    }
    fn handle_proxy_result<T>(&self, result: Result<T, types::Error>) -> Option<T> {
        match result {
            Err(err) => {
                log::error!("Handle Result Error: {}", err.code);
                None
            }
            Ok(t) => Some(t),
        }
    }
}
