use std::sync::OnceLock;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/app/drey/paper-plane/ui/components/animated_bin.ui")]
    pub(crate) struct AnimatedBin {
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AnimatedBin {
        const NAME: &'static str = "PaplAnimatedBin";
        type Type = super::AnimatedBin;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AnimatedBin {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecObject::builder::<gtk::Widget>("child")
                    .read_only()
                    .build()]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "child" => self.obj().child().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.stack.connect_transition_running_notify(|stack| {
                if !stack.is_transition_running() {
                    let mut child = stack.first_child();
                    while let Some(child_) = child {
                        child = child_.next_sibling();
                        if stack.visible_child().as_ref() != Some(&child_) {
                            stack.remove(&child_);
                        }
                    }
                }
            });
        }

        fn dispose(&self) {
            self.stack.unparent();
        }
    }

    impl WidgetImpl for AnimatedBin {}

    #[gtk::template_callbacks]
    impl AnimatedBin {
        #[template_callback]
        fn on_stack_notify_visible_child(&self) {
            self.obj().notify("child");
        }
    }
}

glib::wrapper! {
    pub(crate) struct AnimatedBin(ObjectSubclass<imp::AnimatedBin>) @extends gtk::Widget;
}

impl AnimatedBin {
    pub(crate) fn child(&self) -> Option<gtk::Widget> {
        self.imp().stack.visible_child()
    }

    pub(crate) fn set_child<W: IsA<gtk::Widget>>(&self, child: &W) {
        self.imp().stack.add_child(child);
        self.imp().stack.set_visible_child(child);
    }
}
