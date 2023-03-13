use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    use crate::components::MessageListView;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    template MessageBase {
        GestureClick {
            button: 3;
            released => on_pressed() swapped;
        }

        GestureLongPress {
            touch-only: true;
            pressed => on_long_pressed() swapped;
        }
    }
    "#)]
    pub(crate) struct MessageBase {}

    #[glib::object_subclass]
    impl ObjectSubclass for MessageBase {
        const NAME: &'static str = "MessageBase";
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
            self.show_message_menu(x, y);
        }

        #[template_callback]
        fn on_long_pressed(&self, x: f64, y: f64) {
            self.show_message_menu(x, y);
        }

        fn show_message_menu(&self, x: f64, y: f64) {
            let obj = self.obj();
            let list_view = obj
                .ancestor(MessageListView::static_type())
                .and_downcast::<MessageListView>()
                .unwrap();
            let (x, y) = obj.translate_coordinates(&list_view, x, y).unwrap();

            obj.activate_action("message-list-view.show-message-menu", Some(&(x, y).into()))
                .unwrap();
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
    type Message: glib::IsA<glib::Object>;

    fn new(message: &Self::Message) -> Self {
        glib::Object::builder().property("message", message).build()
    }

    fn message(&self) -> Self::Message {
        self.property("message")
    }

    fn set_message(&self, message: Self::Message);
}

pub(crate) trait MessageBaseImpl: WidgetImpl + ObjectImpl + 'static {}

unsafe impl<T: MessageBaseImpl> IsSubclassable<T> for MessageBase {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
