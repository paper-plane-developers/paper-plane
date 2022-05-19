use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use tdlib::functions;

use crate::session::content::{ChatActionBar, ChatInfoDialog, ItemRow};
use crate::tdlib::{Chat, ChatHistoryItem, ChatType, SponsoredMessage};
use crate::utils::spawn;
use crate::{expressions, Session};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-history.ui")]
    pub(crate) struct ChatHistory {
        pub(super) compact: Cell<bool>,
        pub(super) chat: RefCell<Option<Chat>>,
        pub(super) message_menu: OnceCell<gtk::PopoverMenu>,
        /// This stores the previous vadjustment value of the scrolled window. This is needed to
        /// mark messages as viewed since the scrolled window lags behind the current vadjustment
        /// value due to the scroll animation. In this way, we can calculate the difference of both
        /// vadjustment values and take that into account when we decide which messages are
        /// considered as being viewed.
        pub(super) prev_vadjument_value: Cell<f64>,
        pub(super) sponsored_message_needs_view: Cell<bool>,
        #[template_child]
        pub(super) window_title: TemplateChild<adw::WindowTitle>,
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
            ItemRow::static_type();
            Self::bind_template(klass);

            klass.install_action("chat-history.view-info", None, move |widget, _, _| {
                widget.open_info_dialog();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatHistory {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        "compact",
                        "Compact",
                        "Wheter a compact view is used or not",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "chat",
                        "Chat",
                        "The chat currently shown",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
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

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "chat" => obj.chat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_expressions();

            let adj = self.list_view.vadjustment().unwrap();
            adj.connect_value_changed(clone!(@weak obj => move |adj| {
                obj.load_older_messages(adj);
                obj.update_viewed_messages(
                    &obj.chat().unwrap(),
                    adj.value() - obj.imp().prev_vadjument_value.replace(adj.value())
                );
            }));
        }
    }

    impl WidgetImpl for ChatHistory {}
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
        glib::Object::new(&[]).expect("Failed to create ChatHistory")
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
            if let Some(chat) = self.chat() {
                spawn(clone!(@weak chat => async move {
                    chat.history().load_older_messages().await;
                }));
            }
        }
    }

    fn open_info_dialog(&self) {
        if let Some(chat) = self.chat() {
            ChatInfoDialog::new(&self.parent_window(), &chat).present();
        }
    }

    fn parent_window(&self) -> Option<gtk::Window> {
        self.root()?.downcast().ok()
    }

    fn request_sponsored_message(&self, session: &Session, chat_id: i64, list: &gio::ListStore) {
        spawn(
            clone!(@weak self as obj, @weak session, @weak list => async move {
                match SponsoredMessage::request(chat_id, &session).await {
                    Ok(sponsored_message) => {
                        list.append(&sponsored_message);
                        obj.imp().sponsored_message_needs_view.set(true);
                    }
                    Err(e) => if e.code != 404 {
                        log::warn!("Failed to request a SponsoredMessage: {:?}", e);
                    },
                }
            }),
        );
    }

    pub(crate) fn message_menu(&self) -> &gtk::PopoverMenu {
        self.imp().message_menu.get_or_init(|| {
            gtk::Builder::from_resource("/com/github/melix99/telegrand/ui/message-menu.ui")
                .object::<gtk::PopoverMenu>("menu")
                .unwrap()
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

        // This will mark all messages being read as soon as a new chat is opened (assuming we
        // always open the chat at the end). In the future we should figure out how to scroll the
        // history to the last read message.
        imp.prev_vadjument_value.set(f32::MIN as f64);
        imp.sponsored_message_needs_view.set(false);

        if let Some(ref chat) = chat {
            // Request sponsored message, if needed
            let chat_history: gio::ListModel = if matches!(chat.type_(), ChatType::Supergroup(supergroup) if supergroup.is_channel())
            {
                let list = gio::ListStore::new(gio::ListModel::static_type());

                // We need to create a list here so that we can append the sponsored message
                // to the chat history in the GtkListView using a GtkFlattenListModel
                let sponsored_message_list = gio::ListStore::new(SponsoredMessage::static_type());
                list.append(&sponsored_message_list);
                self.request_sponsored_message(&chat.session(), chat.id(), &sponsored_message_list);

                list.append(chat.history());

                gtk::FlattenListModel::new(Some(&list)).upcast()
            } else {
                chat.history().to_owned().upcast()
            };

            let selection = gtk::NoSelection::new(Some(&chat_history));
            imp.list_view.set_model(Some(&selection));
        }

        imp.chat.replace(chat);
        self.notify("chat");

        let adj = imp.list_view.vadjustment().unwrap();
        self.load_older_messages(&adj);
    }

    fn update_viewed_messages(&self, chat: &Chat, vadjustment_delta: f64) {
        if vadjustment_delta <= 0.0 {
            // We don't really need to check for messages being viewed if the user has scrolled up.
            return;
        }

        let imp = self.imp();

        let chat_id = chat.id();
        let client_id = chat.session().client_id();

        if chat.unread_count() > 0 {
            // Set the bottommost regular message as viewed if necessary.
            view_message(
                self.find_bottommost_message_to_view(chat, vadjustment_delta),
                chat_id,
                client_id,
            );
        }

        if imp.sponsored_message_needs_view.get() {
            let sponsored_message_id =
                visible_sponsored_message_id(&*imp.list_view, vadjustment_delta);
            imp.sponsored_message_needs_view
                .set(sponsored_message_id.is_none());

            // Set the sponsored message as viewed if necessary.
            view_message(sponsored_message_id, chat_id, client_id);
        }
    }

    /// Finds the id of the bottommost regular message that needs to be marked as being viewed.
    /// Returns `None` if currently no new message needs to be marked as being viewed.
    fn find_bottommost_message_to_view(&self, chat: &Chat, vadjustment_delta: f64) -> Option<i64> {
        struct Iter(Option<gtk::Widget>);
        impl Iterator for Iter {
            type Item = gtk::Widget;

            fn next(&mut self) -> Option<Self::Item> {
                let r = self.0.take();
                self.0 = r.as_ref().and_then(|widget| widget.next_sibling());
                r
            }
        }

        let list_view = &*self.imp().list_view;
        Iter(list_view.first_child())
            .find_map(|widget| {
                let maybe_message_viewed_info = check_message_viewed(
                    &widget.first_child().unwrap().downcast::<ItemRow>().unwrap(),
                    list_view,
                    vadjustment_delta,
                );

                maybe_message_viewed_info.and_then(|message_viewed_info| {
                    if message_viewed_info.message_id <= chat.last_read_inbox_message_id() {
                        Some(None)
                    } else if message_viewed_info.considered_as_viewed {
                        Some(Some(message_viewed_info.message_id))
                    } else {
                        None
                    }
                })
            })
            .flatten()
    }
}

/// Struct to indicate whether a message is considered as being viewed.
struct MessageViewedInfo {
    /// The id of the message that may be visible.
    message_id: i64,
    /// Whether the message can be marked as viewed.
    considered_as_viewed: bool,
}

/// Returns a `MessageViewedInfo` for the specified `ItemRow`. Returns `None` if the item row
/// doesn't contain a message.
/// A message is considered as being viewed if parts of it are either visible inside the specified
/// list view or if the user needs to scroll up in order to see it.
fn check_message_viewed(
    item_row: &ItemRow,
    list_view: &gtk::ListView,
    vadjustment_delta: f64,
) -> Option<MessageViewedInfo> {
    item_row
        .item()
        .unwrap()
        .downcast_ref::<ChatHistoryItem>()
        .and_then(ChatHistoryItem::message)
        .map(|message| {
            let (_, dest_y_top) = item_row
                .translate_coordinates(list_view, 0.0, -vadjustment_delta)
                .unwrap();

            MessageViewedInfo {
                message_id: message.id(),
                considered_as_viewed: dest_y_top < list_view.height() as f64,
            }
        })
}

/// Returns the message id of the sponsored message if it visible inside the specified list view.
/// Returns `None` if there is either no sponsored message or it is not visible.
/// A sponsored message (S) is considered visible inside the list view (L) if `S ∩ L = S` applies.
fn visible_sponsored_message_id(list_view: &gtk::ListView, vadjustment_delta: f64) -> Option<i64> {
    list_view
        .first_child()
        .unwrap()
        .first_child()
        .and_then(|child| {
            let item_row = child.downcast::<ItemRow>().unwrap();

            item_row
                .item()
                .unwrap()
                .downcast_ref::<SponsoredMessage>()
                .and_then(|sponsored_message| {
                    let (_, dest_y_top) = item_row
                        .translate_coordinates(list_view, 0.0, -vadjustment_delta)
                        .unwrap();

                    if dest_y_top + (item_row.height() as f64) < list_view.height() as f64
                        && dest_y_top > 0.0
                    {
                        Some(sponsored_message.message_id())
                    } else {
                        None
                    }
                })
        })
}

fn view_message(maybe_message_id: Option<i64>, chat_id: i64, client_id: i32) {
    if let Some(message_id) = maybe_message_id {
        spawn(async move {
            functions::view_messages(chat_id, 0, vec![message_id], true, client_id)
                .await
                .unwrap();
        });
    }
}
