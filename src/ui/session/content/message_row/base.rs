use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::ui;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/message_row/base.ui")]
    pub(crate) struct MessageBase {}

    #[glib::object_subclass]
    impl ObjectSubclass for MessageBase {
        const NAME: &'static str = "PaplMessageBase";
        const ABSTRACT: bool = true;
        type Type = super::MessageBase;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl MessageBase {
        #[template_callback]
        fn on_pressed(&self, _n_press: i32, x: f64, y: f64) {
            self.show_message_menu(x as i32, y as i32);
        }

        #[template_callback]
        fn on_long_pressed(&self, x: f64, y: f64) {
            self.show_message_menu(x as i32, y as i32);
        }

        fn show_message_menu(&self, x: i32, y: i32) {
            let obj = &*self.obj();
            let chat_history = utils::ancestor::<_, ui::ChatHistory>(obj);
            let menu = chat_history.message_menu();

            menu.set_pointing_to(Some(&gdk::Rectangle::new(x, y, 0, 0)));
            menu.unparent();
            menu.set_parent(obj);
            menu.popup();
        }
    }

    impl ObjectImpl for MessageBase {
        fn dispose(&self) {
            let mut child = self.obj().first_child();
            while let Some(child_) = child {
                child = child_.next_sibling();
                child_.unparent();
            }
        }
    }

    impl WidgetImpl for MessageBase {}
}

glib::wrapper! {
    pub(crate) struct MessageBase(ObjectSubclass<imp::MessageBase>)
        @extends gtk::Widget;
}

pub(crate) trait MessageBaseExt:
    glib::object::IsClass + IsA<glib::Object> + IsA<gtk::Widget> + IsA<MessageBase>
{
    type Message: IsA<glib::Object>;

    fn new(message: &Self::Message) -> Self {
        glib::Object::builder().property("message", message).build()
    }

    fn set_message(&self, message: &Self::Message);
}

pub(crate) trait MessageBaseImpl: WidgetImpl + ObjectImpl + 'static {}

unsafe impl<T: MessageBaseImpl> IsSubclassable<T> for MessageBase {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
