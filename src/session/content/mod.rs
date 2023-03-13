mod chat_action_bar;
mod chat_history;
mod chat_info_window;
mod pinned_messages_bar;
mod pinned_messages_view;
mod send_photo_dialog;

use self::chat_action_bar::ChatActionBar;
use self::chat_history::ChatHistory;
use self::chat_info_window::ChatInfoWindow;
use self::pinned_messages_bar::PinnedMessagesBar;
use self::pinned_messages_view::PinnedMessagesView;
use self::send_photo_dialog::SendPhotoDialog;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::tdlib::Chat;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content.ui")]
    pub(crate) struct Content {
        pub(super) compact: Cell<bool>,
        pub(super) chat: RefCell<Option<Chat>>,
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) unselected_chat_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) chat_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub(super) chat_history: TemplateChild<ChatHistory>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Content {
        const NAME: &'static str = "Content";
        type Type = super::Content;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("content.go-back", None, move |widget, _, _| {
                widget.go_back();
            });
            klass.install_action("content.show-pinned-messages", None, move |widget, _, _| {
                widget.show_pinned_messages();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Content {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("compact").build(),
                    glib::ParamSpecObject::builder::<Chat>("chat")
                        .explicit_notify()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "compact" => {
                    let compact = value.get().unwrap();
                    self.compact.set(compact);
                }
                "chat" => {
                    let chat = value.get().unwrap();
                    obj.set_chat(chat);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Content {}
    impl BinImpl for Content {}
}

glib::wrapper! {
    pub(crate) struct Content(ObjectSubclass<imp::Content>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Content {
    fn default() -> Self {
        Self::new()
    }
}

impl Content {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn handle_paste_action(&self) {
        self.imp().chat_history.handle_paste_action();
    }

    fn go_back(&self) {
        self.imp()
            .chat_leaflet
            .navigate(adw::NavigationDirection::Back);
    }

    fn show_pinned_messages(&self) {
        if let Some(chat) = self.chat() {
            let imp = self.imp();

            let next_child = imp
                .chat_leaflet
                .adjacent_child(adw::NavigationDirection::Forward);
            let cached = if let Some(pinned_messages_view) =
                next_child.and_downcast::<PinnedMessagesView>()
            {
                pinned_messages_view.chat() == chat
            } else {
                false
            };

            if !cached {
                let pinned_messages = PinnedMessagesView::new(&chat);
                imp.chat_leaflet.append(&pinned_messages);
            }

            imp.chat_leaflet.navigate(adw::NavigationDirection::Forward);
        }
    }

    pub(crate) fn chat(&self) -> Option<Chat> {
        self.imp().chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let imp = self.imp();
        if chat.is_some() {
            // Remove every leaflet page except the first one (the first chat history)
            imp.chat_leaflet
                .pages()
                .iter::<adw::LeafletPage>()
                .map(|p| p.unwrap())
                .enumerate()
                .filter(|(i, _)| i > &0)
                .for_each(|(_, p)| imp.chat_leaflet.remove(&p.child()));

            imp.stack.set_visible_child(&imp.chat_leaflet.get());
        } else {
            imp.stack.set_visible_child(&imp.unselected_chat_view.get());
        }

        imp.chat.replace(chat);

        self.notify("chat");
    }
}
