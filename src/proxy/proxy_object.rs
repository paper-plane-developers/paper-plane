use glib::Object;
use glib::ParamFlags;
use glib::ParamSpec;
use glib::Value;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::rc::Rc;
use tdgrand::enums::ProxyType;
use tdgrand::types::Proxy;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ProxyObject {
        pub proxy: Rc<RefCell<Proxy>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProxyObject {
        const NAME: &'static str = "ProxyObject";
        type Type = super::ProxyObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ProxyObject {
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
                    ParamSpec::new_string(
                        "type",
                        "type",
                        "type",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpec::new_string(
                        "server",
                        "server",
                        "server",
                        None,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "enabled" => {
                    let input_value = value.get().expect("The value needs to be of type `bool`.");
                    self.proxy.borrow_mut().is_enabled = input_value;
                },
                "type" => {},
                "server" => {}
                _ => unimplemented!(),
            }
        }
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "enabled" => self.proxy.borrow().is_enabled.to_value(),
                "type" => match self.proxy.borrow().r#type {
                    ProxyType::Http(_) => "HTTP  ".to_value(),
                    ProxyType::Socks5(_) => "SOCKS5".to_value(),
                    _ => unimplemented!(),
                },
                "server" => format!("{}:{}", self.proxy.borrow().server, self.proxy.borrow().port).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct ProxyObject(ObjectSubclass<imp::ProxyObject>);
}

impl ProxyObject {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create ProxyObject.")
    }

    pub fn set_proxy(&self, proxy: Proxy) {
        let self_ = imp::ProxyObject::from_instance(self);
        self_.proxy.replace(proxy);
    }
}
