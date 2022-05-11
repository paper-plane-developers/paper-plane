use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    use crate::session::content::ChatHistory;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="ContentMessageBase" parent="GtkWidget">
        <child>
          <object class="GtkGestureClick">
            <property name="button">3</property>
            <signal name="released" handler="on_pressed" swapped="true"/>
          </object>
        </child>
        <child>
          <object class="GtkGestureLongPress">
            <property name="touch-only">True</property>
            <signal name="pressed" handler="on_pressed" swapped="true"/>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    pub(crate) struct MessageBase {}

    #[glib::object_subclass]
    impl ObjectSubclass for MessageBase {
        const NAME: &'static str = "ContentMessageBase";
        const ABSTRACT: bool = true;
        type Type = super::MessageBase;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl MessageBase {
        #[template_callback]
        fn on_pressed(&self) {
            self.show_message_menu();
        }

        fn show_message_menu(&self) {
            let obj = self.instance();
            let chat_history = obj.ancestor(ChatHistory::static_type()).unwrap();
            let menu = chat_history
                .downcast_ref::<ChatHistory>()
                .unwrap()
                .message_menu();

            menu.unparent();
            menu.set_parent(&obj);
            menu.popup();
        }
    }

    impl ObjectImpl for MessageBase {
        fn dispose(&self, obj: &Self::Type) {
            let mut child = obj.first_child();
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

pub(crate) trait MessageBaseImpl: WidgetImpl + ObjectImpl + 'static {}

unsafe impl<T: MessageBaseImpl> IsSubclassable<T> for MessageBase {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
