use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use std::sync::mpsc;

use crate::add_account_window::AddAccountWindow;
use crate::telegram;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use glib::subclass;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/window.ui")]
    pub struct TelegrandWindow {
        #[template_child]
        pub add_account_window: TemplateChild<AddAccountWindow>,
        #[template_child]
        pub chat_stack: TemplateChild<gtk::Stack>,
    }

    impl ObjectSubclass for TelegrandWindow {
        const NAME: &'static str = "TelegrandWindow";
        type Type = super::TelegrandWindow;
        type ParentType = adw::ApplicationWindow;
        type Interfaces = ();
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                add_account_window: TemplateChild::default(),
                chat_stack: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TelegrandWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for TelegrandWindow {}
    impl WindowImpl for TelegrandWindow {}
    impl ApplicationWindowImpl for TelegrandWindow {}
    impl AdwApplicationWindowImpl for TelegrandWindow {}
}

glib::wrapper! {
    pub struct TelegrandWindow(ObjectSubclass<imp::TelegrandWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl TelegrandWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P, gtk_receiver: glib::Receiver<telegram::MessageGTK>, tg_sender: mpsc::Sender<telegram::MessageTG>) -> Self {
        let window: Self = glib::Object::new(&[("application", app)])
            .expect("Failed to create TelegrandWindow");

        let self_ = imp::TelegrandWindow::from_instance(&window);
        let add_account_window = &*self_.add_account_window;
        add_account_window.init_signals(&tg_sender);

        let chat_stack = &*self_.chat_stack;
        gtk_receiver.attach(None, glib::clone!(@weak add_account_window, @weak chat_stack => move |msg| {
            match msg {
                telegram::MessageGTK::AccountNotAuthorized =>
                    add_account_window.show(),
                telegram::MessageGTK::NeedConfirmationCode =>
                    add_account_window.navigate_forward(),
                telegram::MessageGTK::SuccessfullySignedIn =>
                    add_account_window.hide(),
                telegram::MessageGTK::LoadChat(chat_id, chat_name) => {
                    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
                    chat_stack.add_titled(&text_box, Some(&chat_id), &chat_name);
                }
                telegram::MessageGTK::NewMessage(chat_id, chat_name, message_text) => {
                    match chat_stack.get_child_by_name(&chat_id) {
                        Some(child) => {
                            let text_box: gtk::Box = child.downcast().unwrap();
                            let label = gtk::Label::new(Some(&message_text));
                            text_box.append(&label);
                        }
                        None => {
                            let text_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
                            let label = gtk::Label::new(Some(&message_text));
                            text_box.append(&label);
                            chat_stack.add_titled(&text_box, Some(&chat_id), &chat_name);
                        }
                    }
                }
            }

            glib::Continue(true)
        }));

        window
    }
}
