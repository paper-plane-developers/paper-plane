use adw::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use tdlib::enums::ChatMemberStatus;
use tdlib::functions;

use crate::session::content::{
    ChatActionBar, ChatHistoryError, ChatHistoryModel, ChatHistoryRow, ChatInfoWindow,
};
use crate::tdlib::{Chat, ChatType, SponsoredMessage};
use crate::utils::spawn;
use crate::{expressions, Session};

const MIN_N_ITEMS: u32 = 20;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};
    use std::collections::HashSet;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-history.ui")]
    pub(crate) struct ChatHistory {
        pub(super) compact: Cell<bool>,
        pub(super) chat: RefCell<Option<Chat>>,
        pub(super) model: RefCell<Option<ChatHistoryModel>>,
        pub(super) message_menu: OnceCell<gtk::PopoverMenu>,
        pub(super) is_auto_scrolling: Cell<bool>,
        pub(super) sticky: Cell<bool>,
        pub(super) visible_messages: RefCell<HashSet<i64>>,
        #[template_child]
        pub(super) window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub(super) list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub(super) chat_action_bar: TemplateChild<ChatActionBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatHistory {
        const NAME: &'static str = "ContentChatHistory";
        type Type = super::ChatHistory;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ChatHistoryRow::static_type();
            klass.bind_template();

            klass.install_action("chat-history.view-info", None, move |widget, _, _| {
                widget.open_info_dialog();
            });
            klass.install_action("chat-history.scroll-down", None, move |widget, _, _| {
                widget.scroll_down();
            });
            klass.install_action(
                "chat-history.reply",
                Some("x"),
                move |widget, _, variant| {
                    let message_id = variant.and_then(|v| v.get()).unwrap();
                    widget.imp().chat_action_bar.reply_to_message_id(message_id);
                },
            );
            klass.install_action("chat-history.edit", Some("x"), move |widget, _, variant| {
                let message_id = variant.and_then(|v| v.get()).unwrap();
                widget.imp().chat_action_bar.edit_message_id(message_id);
            });
            klass.install_action_async(
                "chat-history.leave-chat",
                None,
                |widget, _, _| async move {
                    widget.show_leave_chat_dialog().await;
                },
            );
            klass.install_action_async(
                "chat-history.add-visible-message",
                Some("x"),
                |widget, _, variant| async move {
                    let message_id = variant.and_then(|v| v.get()).unwrap();
                    if widget
                        .imp()
                        .visible_messages
                        .borrow_mut()
                        .insert(message_id)
                    {
                        println!("ADD {}", message_id);
                        widget.update_visible_messages().await;
                    }
                },
            );
            klass.install_action_async(
                "chat-history.remove-visible-message",
                Some("x"),
                |widget, _, variant| async move {
                    let message_id = variant.and_then(|v| v.get()).unwrap();
                    if widget
                        .imp()
                        .visible_messages
                        .borrow_mut()
                        .remove(&message_id)
                    {
                        println!("REMOVE {}", message_id);
                        widget.update_visible_messages().await;
                    }
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatHistory {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("compact").build(),
                    glib::ParamSpecObject::builder::<Chat>("chat")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecBoolean::builder("sticky")
                        .read_only()
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
                "sticky" => obj.set_sticky(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "chat" => obj.chat().to_value(),
                "sticky" => obj.sticky().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.setup_expressions();

            let adj = self.list_view.vadjustment().unwrap();
            adj.connect_value_changed(clone!(@weak obj => move |adj| {
                let imp = obj.imp();

                if imp.is_auto_scrolling.get() {
                    if adj.value() + adj.page_size() >= adj.upper() {
                        imp.is_auto_scrolling.set(false);
                        obj.set_sticky(true);
                    }
                } else {
                    obj.set_sticky(adj.value() + adj.page_size() >= adj.upper());
                    obj.load_older_messages(adj);
                }
            }));

            adj.connect_upper_notify(clone!(@weak obj => move |_| {
                if obj.sticky() || obj.imp().is_auto_scrolling.get() {
                    obj.scroll_down();
                }
            }));
        }
    }

    impl WidgetImpl for ChatHistory {
        fn direction_changed(&self, previous_direction: gtk::TextDirection) {
            let obj = self.obj();

            if obj.direction() == previous_direction {
                return;
            }

            if let Some(menu) = self.message_menu.get() {
                menu.set_halign(if obj.direction() == gtk::TextDirection::Rtl {
                    gtk::Align::End
                } else {
                    gtk::Align::Start
                });
            }
        }
    }

    impl BinImpl for ChatHistory {}
}

glib::wrapper! {
    pub(crate) struct ChatHistory(ObjectSubclass<imp::ChatHistory>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatHistory {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    fn setup_expressions(&self) {
        let chat_expression = Self::this_expression("chat");

        // Chat title
        expressions::chat_display_name(&chat_expression).bind(
            &*self.imp().window_title,
            "title",
            Some(self),
        );
    }

    fn load_older_messages(&self, adj: &gtk::Adjustment) {
        if adj.value() < adj.page_size() * 2.0 || adj.upper() <= adj.page_size() * 2.0 {
            if let Some(model) = self.imp().model.borrow().as_ref() {
                spawn(clone!(@weak model => async move {
                    if let Err(ChatHistoryError::Tdlib(e)) = model.load_older_messages(20).await {
                        log::warn!("Couldn't load more chat messages: {:?}", e);
                    }
                }));
            }
        }
    }

    async fn update_visible_messages(&self) {
        if let Some(chat) = self.chat() {
            let client_id = chat.session().client_id();
            let message_ids = self
                .imp()
                .visible_messages
                .borrow()
                .clone()
                .into_iter()
                .collect();
            let result =
                tdlib::functions::view_messages(chat.id(), 0, message_ids, false, client_id).await;

            if let Err(e) = result {
                log::warn!("Error setting visible messages: {e:?}");
            }

            let msgs: Vec<String> = self
                .imp()
                .visible_messages
                .borrow()
                .iter()
                .map(|id| {
                    let message = self.chat().unwrap().message(*id).unwrap();
                    format!("{} ||| {}", crate::strings::message_content(&message), id)
                })
                .collect();
            dbg!(msgs);
            println!();
            println!();
        }
    }

    fn open_info_dialog(&self) {
        if let Some(chat) = self.chat() {
            ChatInfoWindow::new(&self.parent_window(), &chat).present();
        }
    }

    async fn show_leave_chat_dialog(&self) {
        if let Some(chat) = self.chat() {
            let dialog = adw::MessageDialog::new(
                Some(&self.parent_window().unwrap()),
                Some(&gettext("Leave chat?")),
                Some(&gettext("Do you want to leave this chat?")),
            );
            dialog.add_responses(&[("no", &gettext("_No")), ("yes", &gettext("_Yes"))]);
            dialog.set_default_response(Some("no"));
            dialog.set_close_response("no");
            dialog.set_response_appearance("yes", adw::ResponseAppearance::Destructive);
            let response = dialog.choose_future().await;
            if response == "yes" {
                let result = functions::leave_chat(chat.id(), chat.session().client_id()).await;
                if let Err(e) = result {
                    log::warn!("Failed to leave chat: {:?}", e);
                } else {
                    // Unselect recently left chat
                    chat.session().imp().sidebar.get().set_selected_chat(None);
                }
            }
        }
    }

    fn parent_window(&self) -> Option<gtk::Window> {
        self.root()?.downcast().ok()
    }

    fn request_sponsored_message(&self, session: &Session, chat_id: i64, list: &gio::ListStore) {
        spawn(clone!(@weak session, @weak list => async move {
            match SponsoredMessage::request(chat_id, &session).await {
                Ok(sponsored_message) => {
                    if let Some(sponsored_message) = sponsored_message {
                        list.append(&sponsored_message);
                    }
                }
                Err(e) => {
                    if e.code != 404 {
                        log::warn!("Failed to request a SponsoredMessage: {:?}", e);
                    }
                }
            }
        }));
    }

    pub(crate) fn message_menu(&self) -> &gtk::PopoverMenu {
        self.imp().message_menu.get_or_init(|| {
            let menu =
                gtk::Builder::from_resource("/com/github/melix99/telegrand/ui/message-menu.ui")
                    .object::<gtk::PopoverMenu>("menu")
                    .unwrap();

            menu.set_halign(if self.direction() == gtk::TextDirection::Rtl {
                gtk::Align::End
            } else {
                gtk::Align::Start
            });

            menu
        })
    }

    pub(crate) fn handle_paste_action(&self) {
        self.imp().chat_action_bar.handle_paste_action();
    }

    pub(crate) fn chat(&self) -> Option<Chat> {
        self.imp().chat.borrow().clone()
    }

    pub(crate) fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        let imp = self.imp();

        if let Some(ref chat) = chat {
            self.action_set_enabled(
                "chat-history.leave-chat",
                match chat.type_() {
                    ChatType::BasicGroup(data) => data.status().0 != ChatMemberStatus::Left,
                    ChatType::Supergroup(data) => data.status().0 != ChatMemberStatus::Left,
                    _ => false,
                },
            );

            let model = ChatHistoryModel::new(chat);

            // Request sponsored message, if needed
            let list_view_model: gio::ListModel = if matches!(chat.type_(), ChatType::Supergroup(supergroup) if supergroup.is_channel())
            {
                let list = gio::ListStore::new(gio::ListModel::static_type());

                // We need to create a list here so that we can append the sponsored message
                // to the chat history in the GtkListView using a GtkFlattenListModel
                let sponsored_message_list = gio::ListStore::new(SponsoredMessage::static_type());
                list.append(&sponsored_message_list);
                self.request_sponsored_message(&chat.session(), chat.id(), &sponsored_message_list);

                list.append(&model);

                gtk::FlattenListModel::new(Some(list)).upcast()
            } else {
                model.clone().upcast()
            };

            spawn(clone!(@weak model => async move {
                while model.n_items() < MIN_N_ITEMS {
                    let limit = MIN_N_ITEMS - model.n_items();
                    match model.load_older_messages(limit as i32).await {
                        Ok(can_load_more) => if !can_load_more {
                            break;
                        }
                        Err(e) => {
                            log::warn!("Couldn't load initial history messages: {}", e);
                            break;
                        }
                    }
                }
            }));

            let selection = gtk::NoSelection::new(Some(list_view_model));
            imp.list_view.set_model(Some(&selection));

            imp.model.replace(Some(model));
        }

        imp.chat.replace(chat);
        self.notify("chat");
    }

    pub(crate) fn sticky(&self) -> bool {
        self.imp().sticky.get()
    }

    fn set_sticky(&self, sticky: bool) {
        if self.sticky() == sticky {
            return;
        }

        self.imp().sticky.set(sticky);
        self.notify("sticky");
    }

    fn scroll_down(&self) {
        let imp = self.imp();

        imp.is_auto_scrolling.set(true);

        imp.scrolled_window
            .emit_by_name::<bool>("scroll-child", &[&gtk::ScrollType::End, &false]);
    }
}
