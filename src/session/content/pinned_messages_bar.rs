use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    template PinnedMessagesBar {
        Box content_box {
            styles ["toolbar"]

            Box {
                orientation: vertical;
                hexpand: true;

                Inscription {

                }

                Inscription {

                }
            }

            Button {
                icon-name: "view-list-symbolic";
                action-name: "content.show-pinned-messages";
            }
        }
    }
    "#)]
    pub(crate) struct PinnedMessagesBar {
        #[template_child]
        pub(super) content_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PinnedMessagesBar {
        const NAME: &'static str = "PinnedMessagesBar";
        type Type = super::PinnedMessagesBar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PinnedMessagesBar {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for PinnedMessagesBar {}
}

glib::wrapper! {
    pub(crate) struct PinnedMessagesBar(ObjectSubclass<imp::PinnedMessagesBar>)
        @extends gtk::Widget;
}
