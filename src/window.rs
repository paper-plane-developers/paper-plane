use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use tokio::runtime;
use tokio::sync::mpsc;

use crate::add_account_window::AddAccountWindow;
use crate::chat_page::ChatPage;
use crate::dialog_data::DialogData;
use crate::dialog_model::DialogModel;
use crate::dialog_row::DialogRow;
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
        pub dialogs_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub chat_stack: TemplateChild<gtk::Stack>,
        pub dialog_model: DialogModel,
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
                dialogs_list: TemplateChild::default(),
                chat_stack: TemplateChild::default(),
                dialog_model: DialogModel::new(),
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
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P, gtk_receiver: glib::Receiver<telegram::EventGTK>, tg_sender: mpsc::Sender<telegram::EventTG>) -> Self {
        let window = glib::Object::new(&[("application", app)])
            .expect("Failed to create TelegrandWindow");

        let self_ = imp::TelegrandWindow::from_instance(&window);
        let add_account_window = &*self_.add_account_window;
        add_account_window.init_signals(&tg_sender);

        let chat_stack = &*self_.chat_stack;
        let dialog_model = self_.dialog_model.clone();
        self_.dialogs_list.connect_row_activated(glib::clone!(@weak chat_stack, @weak dialog_model => move |_, row| {
            let index = row.get_index();
            if let Some(item) = dialog_model.get_object(index as u32) {
                let data = item.downcast_ref::<DialogData>()
                    .expect("Row data is of wrong type");
                let chat_id = data.get_chat_id();
                chat_stack.set_visible_child_name(&chat_id);
            }
        }));

        self_.dialogs_list.bind_model(Some(&self_.dialog_model), move |item| {
            let data = item.downcast_ref::<DialogData>()
                .expect("Row data is of wrong type");
            let row = DialogRow::new(data);
            row.upcast::<gtk::Widget>()
        });

        gtk_receiver.attach(None, glib::clone!(@weak window, @weak add_account_window, @weak chat_stack, @weak dialog_model => move |msg| {
            match msg {
                telegram::EventGTK::AccountNotAuthorized =>
                    add_account_window.show(),
                telegram::EventGTK::NeedConfirmationCode =>
                    add_account_window.navigate_forward(),
                telegram::EventGTK::AccountAuthorized => {
                    add_account_window.hide();

                    let _ = runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(
                            tg_sender.send(telegram::EventTG::RequestDialogs));
                }
                telegram::EventGTK::ReceivedDialog(dialog) => {
                    let chat = dialog.chat();
                    let chat_id = chat.id().to_string();
                    let chat_name = chat.name().to_string();
                    let last_message = dialog.last_message.as_ref().unwrap().text();
                    dialog_model.append(&DialogData::new(&chat_id, &chat_name,
                        last_message));

                    let chat_page = ChatPage::new(&tg_sender, dialog);
                    chat_stack.add_titled(&chat_page, Some(&chat_id), &chat_name);
                }
                telegram::EventGTK::NewMessage(message) => {
                    if !message.outgoing() {
                        let chat = message.chat();
                        let chat_name = chat.name();
                        let message_text = message.text();

                        let notification = gio::Notification::new("Telegrand");
                        notification.set_title(chat_name);
                        notification.set_body(Some(message_text));
                        let app = window.get_application().unwrap();
                        app.send_notification(Some("new-message"), &notification);
                    }
                }
            }

            glib::Continue(true)
        }));

        window
    }
}
