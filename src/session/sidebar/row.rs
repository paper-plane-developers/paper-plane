use gettextrs::gettext;
use glib::{clone, closure};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};
use tdlib::enums::{InputMessageContent, MessageContent, MessageSendingState};
use tdlib::types::DraftMessage;

use crate::tdlib::{
    BoxedChatNotificationSettings, BoxedDraftMessage, BoxedMessageContent,
    BoxedScopeNotificationSettings, Chat, ChatAction, ChatActionList, ChatType, Message,
    MessageForwardInfo, MessageForwardOrigin,
};
use crate::utils::{dim_and_escape, escape, spawn};
use crate::{expressions, strings, Session};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use crate::session::sidebar::mini_thumbnail::MiniThumbnail;
    use crate::session::sidebar::{Avatar, Sidebar};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-row.ui")]
    pub(crate) struct Row {
        pub(super) item: RefCell<Option<Chat>>,
        pub(super) bindings: RefCell<Vec<gtk::ExpressionWatch>>,
        pub(super) chat_notify_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        #[template_child]
        pub(super) title_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) message_status_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) timestamp_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) message_prefix_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) message_thumbnail: TemplateChild<MiniThumbnail>,
        #[template_child]
        pub(super) bottom_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) status_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) empty_status_bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) pin_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) unread_mention_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) unread_count_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "SidebarRow";
        type Type = super::Row;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("sidebar-row.pin", None, move |widget, _, _| {
                widget.toggle_chat_is_pinned()
            });
            klass.install_action("sidebar-row.unpin", None, move |widget, _, _| {
                widget.toggle_chat_is_pinned()
            });
            klass.install_action("sidebar-row.mark-as-unread", None, move |widget, _, _| {
                widget.toggle_chat_marked_as_unread()
            });
            klass.install_action("sidebar-row.mark-as-read", None, move |widget, _, _| {
                widget.toggle_chat_marked_as_unread()
            });

            Avatar::static_type();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl Row {
        #[template_callback]
        fn on_pressed(&self) {
            self.show_menu();
        }

        fn show_menu(&self) {
            let obj = self.obj();
            let sidebar = obj.ancestor(Sidebar::static_type()).unwrap();
            let menu = sidebar.downcast_ref::<Sidebar>().unwrap().row_menu();

            menu.unparent();
            menu.set_parent(&*obj);
            menu.popup();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<glib::Object>("item")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.set_item(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.item().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.message_prefix_label
                .connect_label_notify(|label| label.set_visible(!label.label().is_empty()));

            self.message_thumbnail
                .connect_notify_local(Some("paintable"), |thumbnail, _| {
                    thumbnail.set_visible(thumbnail.paintable().is_some())
                });

            self.obj().setup_expressions();
        }

        fn dispose(&self) {
            let mut child = self.obj().first_child();
            while let Some(child_) = child {
                child = child_.next_sibling();
                child_.unparent();
            }
        }
    }

    impl WidgetImpl for Row {}
}

glib::wrapper! {
    pub(crate) struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Row {
    pub(crate) fn new() -> Self {
        glib::Object::builder().build()
    }

    fn toggle_chat_is_pinned(&self) {
        if let Some(chat) = self.item() {
            spawn(async move {
                if let Err(e) = chat.toggle_is_pinned().await {
                    log::warn!("Error on toggling chat's pinned state: {e:?}");
                }
            });
        }
    }

    fn toggle_chat_marked_as_unread(&self) {
        if let Some(chat) = self.item() {
            spawn(async move {
                if let Err(e) = if chat.unread_count() > 0 || chat.is_marked_as_unread() {
                    chat.mark_as_read().await
                } else {
                    chat.mark_as_unread().await
                } {
                    log::warn!("Error on toggling chat's unread state: {e:?}");
                }
            });
        }
    }

    fn setup_expressions(&self) {
        let imp = self.imp();
        let item_expression = Self::this_expression("item");

        // Chat title
        expressions::chat_display_name(&item_expression).bind(
            &*imp.title_label,
            "text",
            Some(self),
        );

        // Chat unread count
        item_expression
            .chain_property::<Chat>("unread-count")
            .chain_closure::<String>(closure!(|_: Self, unread_count: i32| {
                if unread_count > 0 {
                    unread_count.to_string()
                } else {
                    String::new()
                }
            }))
            .bind(&*imp.unread_count_label, "label", Some(self));

        // Decide what to show (exclusive): pin icon, unread label, unread count or marked as unread
        // bin.
        gtk::ClosureExpression::new::<gtk::Widget>(
            &[
                item_expression.chain_property::<Chat>("is-pinned").upcast(),
                item_expression
                    .chain_property::<Chat>("unread-count")
                    .upcast(),
                item_expression
                    .chain_property::<Chat>("unread-mention-count")
                    .upcast(),
                item_expression
                    .chain_property::<Chat>("is_marked_as_unread")
                    .upcast(),
            ],
            closure!(|obj: Self,
                      is_pinned: bool,
                      unread_count: i32,
                      unread_mention: i32,
                      is_marked_as_unread: bool| {
                let imp = obj.imp();

                let visible_child: &gtk::Widget = if unread_mention > 0 {
                    imp.unread_mention_label.upcast_ref()
                } else if unread_count > 0 || is_marked_as_unread {
                    imp.unread_count_label.upcast_ref()
                } else if is_pinned {
                    imp.pin_icon.upcast_ref()
                } else {
                    imp.empty_status_bin.upcast_ref()
                };

                visible_child.clone()
            }),
        )
        .bind(&*imp.status_stack, "visible-child", Some(self));
    }

    pub(crate) fn item(&self) -> Option<Chat> {
        self.imp().item.borrow().to_owned()
    }

    pub(crate) fn set_item(&self, item: Option<Chat>) {
        if self.item() == item {
            return;
        }

        let imp = self.imp();
        let mut bindings = imp.bindings.borrow_mut();

        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        if let Some(handler_id) = imp.chat_notify_handler_id.take() {
            self.item()
                .unwrap()
                .disconnect(handler_id);
        }

        if let Some(ref chat) = item {
                let last_message_expression = Chat::this_expression("last-message");
                let draft_message_expression = Chat::this_expression("draft-message");
                let actions_expression = Chat::this_expression("actions");
                let notification_settings_expression =
                    Chat::this_expression("notification-settings");
                let session_expression = Chat::this_expression("session");

                // Message status bindings
                if !chat.is_own_chat() {
                    let message_status_visibility_binding = last_message_expression
                        .chain_property::<Message>("is-outgoing")
                        .bind(&*imp.message_status_icon, "visible", Some(chat));
                    bindings.push(message_status_visibility_binding);

                    let message_status_icon_binding = gtk::ClosureExpression::new::<String>(
                        [
                            last_message_expression.upcast_ref(),
                            Chat::this_expression("last-read-outbox-message-id").upcast_ref(),
                        ],
                        closure!(|_: Chat,
                                  last_message: Option<Message>,
                                  last_read_msg_id: i64| {
                            last_message
                                .filter(Message::is_outgoing)
                                .map(|message| match message.sending_state() {
                                    Some(state) => match state.0 {
                                        MessageSendingState::Failed(_) => "message-failed-symbolic",
                                        MessageSendingState::Pending => "message-pending-symbolic",
                                    },
                                    None => {
                                        if message.id() == last_read_msg_id {
                                            "message-read-symbolic"
                                        } else {
                                            "message-unread-right-symbolic"
                                        }
                                    }
                                })
                                .unwrap_or("")
                        }),
                    )
                    .bind(&*imp.message_status_icon, "icon-name", Some(chat));
                    bindings.push(message_status_icon_binding);

                    let message_status_css_binding = last_message_expression
                        .chain_closure::<Vec<String>>(closure!(
                            |_: Chat, last_message: Option<Message>| {
                                last_message
                                    .filter(Message::is_outgoing)
                                    .map(|message| {
                                        vec![match message.sending_state() {
                                            Some(state) => match state.0 {
                                                MessageSendingState::Failed(_) => "error",
                                                MessageSendingState::Pending => "dim-label",
                                            },
                                            None => "accent",
                                        }
                                        .to_string()]
                                    })
                                    .unwrap_or_default()
                            }
                        ))
                        .bind(&*imp.message_status_icon, "css-classes", Some(chat));
                    bindings.push(message_status_css_binding);
                } else {
                    imp.message_status_icon.set_visible(false);
                    imp.message_status_icon.set_icon_name(None);
                    imp.message_status_icon.set_css_classes(&[]);
                }

                // Timestamp label bindings
                let timestamp_binding = gtk::ClosureExpression::new::<i32>(
                    &[
                        draft_message_expression.clone().upcast(),
                        last_message_expression.clone().upcast(),
                    ],
                    closure!(|_: Chat,
                              draft_message: Option<BoxedDraftMessage>,
                              last_message: Option<Message>| {
                        draft_message.map(|m| m.0.date).unwrap_or_else(|| {
                            // ... Or, if there is no draft message use the timestamp of the
                            // last message.
                            last_message.map(|m| m.date()).unwrap_or_default()
                        })
                    }),
                )
                .chain_closure::<glib::GString>(closure!(|_: Chat, date: i32| {
                    let datetime_now = glib::DateTime::now_local().unwrap();
                    let datetime = glib::DateTime::from_unix_utc(date as i64)
                        .and_then(|t| t.to_local())
                        .unwrap();

                    let difference = datetime_now.difference(&datetime);
                    let hours_difference = difference.as_hours();
                    let days_difference = difference.as_days();

                    if hours_difference <= 16 {
                        datetime.format(&gettext("%l:%M %p")).unwrap()
                    } else if days_difference < 6 {
                        // Show the day of the week
                        datetime.format("%a").unwrap()
                    } else if days_difference < 364 {
                        // Show the day and the month
                        datetime.format("%d %b").unwrap()
                    } else {
                        // Show the entire date
                        datetime.format("%x").unwrap()
                    }
                }))
                .bind(&*imp.timestamp_label, "label", Some(chat));
                bindings.push(timestamp_binding);

                // Actions, draft message and last message bindings.
                let message_prefix_binding = gtk::ClosureExpression::new::<String>(
                    [
                        actions_expression.upcast_ref(),
                        draft_message_expression.upcast_ref(),
                        last_message_expression.upcast_ref(),
                    ],
                    closure!(|_: Chat,
                              actions: ChatActionList,
                              draft_message: Option<BoxedDraftMessage>,
                              message: Option<Message>| {
                        actions
                            .last()
                            .map(|_| String::new())
                            .or_else(|| {
                                draft_message.map(|_| {
                                    format!(
                                        "<span foreground=\"#e01b24\">{}:</span>",
                                        gettext("Draft"),
                                    )
                                })
                            })
                            .or_else(|| {
                                message
                                    .and_then(sender_label)
                                    .map(|sender_label| format!("{sender_label}:"))
                            })
                    }),
                )
                .bind(&*imp.message_prefix_label, "label", Some(chat));
                bindings.push(message_prefix_binding);

                let thumbnail_paintable_binding =
                    gtk::ClosureExpression::new::<Option<gdk::Texture>>(
                        [
                            actions_expression.upcast_ref(),
                            draft_message_expression.upcast_ref(),
                            last_message_expression.upcast_ref(),
                        ],
                        closure!(|_: Chat,
                                  actions: ChatActionList,
                                  draft_message: Option<BoxedDraftMessage>,
                                  message: Option<Message>| {
                            if actions.n_items() > 0 || draft_message.is_some() {
                                None
                            } else {
                                message.and_then(message_thumbnail_texture)
                            }
                        }),
                    )
                    .bind(&*imp.message_thumbnail, "paintable", Some(chat));
                bindings.push(thumbnail_paintable_binding);

                let content_expression =
                    last_message_expression.chain_property::<Message>("content");

                let message_binding = gtk::ClosureExpression::new::<String>(
                    &[
                        // TODO: In the future, consider making this a bit more efficient: We
                        // sometimes don't need to update if for example an action was removed that
                        // was not in the group of recent actions.
                        actions_expression.upcast(),
                        draft_message_expression.upcast(),
                        last_message_expression.upcast(),
                        content_expression.upcast(),
                    ],
                    closure!(|_: Chat,
                              actions: ChatActionList,
                              draft_message: Option<BoxedDraftMessage>,
                              last_message: Option<Message>,
                              _content: BoxedMessageContent| {
                        actions
                            .last()
                            .map(|action| {
                                format!(
                                    "<span foreground=\"#3584e4\">{}</span>",
                                    stringify_action(&action)
                                )
                            })
                            .or_else(|| {
                                draft_message.map(|m| dim_and_escape(&draft_message_text(m.0)))
                            })
                            .or_else(|| {
                                last_message.map(|message| {
                                    dim_and_escape(&strings::message_content(&message))
                                })
                            })
                            .unwrap_or_default()
                    }),
                )
                .bind(&*imp.bottom_label, "markup", Some(chat));
                bindings.push(message_binding);

                // Unread count css classes binding
                let scope_notification_settings_expression = session_expression
                    .chain_property::<Session>(match chat.type_() {
                        ChatType::Private(_) | ChatType::Secret(_) => {
                            "private-chats-notification-settings"
                        }
                        ChatType::BasicGroup(_) => "group-chats-notification-settings",
                        ChatType::Supergroup(supergroup) => {
                            if supergroup.is_channel() {
                                "channel-chats-notification-settings"
                            } else {
                                "group-chats-notification-settings"
                            }
                        }
                    });
                let unread_binding = gtk::ClosureExpression::new::<Vec<String>>(
                    &[
                        notification_settings_expression.upcast(),
                        scope_notification_settings_expression.upcast(),
                    ],
                    closure!(|_: Chat,
                              notification_settings: BoxedChatNotificationSettings,
                              scope_notification_settings: Option<
                        BoxedScopeNotificationSettings,
                    >| {
                        vec![
                            "unread-count".to_string(),
                            if notification_settings.0.use_default_mute_for {
                                if scope_notification_settings
                                    .map(|s| s.0.mute_for > 0)
                                    .unwrap_or(notification_settings.0.mute_for > 0)
                                {
                                    "unread-count-muted"
                                } else {
                                    "unread-count-unmuted"
                                }
                            } else if notification_settings.0.mute_for > 0 {
                                "unread-count-muted"
                            } else {
                                "unread-count-unmuted"
                            }
                            .to_string(),
                        ]
                    }),
                )
                .bind(&*imp.unread_count_label, "css-classes", Some(chat));
                bindings.push(unread_binding);
        }

        self.update_actions_for_item(item.as_ref());
        imp.item.replace(item);
        self.notify("item");
    }

    fn update_actions_for_item(&self, item: Option<&Chat>) {
        if let Some(chat) = item {
                self.update_actions_for_chat(chat);
                let handler_id = chat.connect_notify_local(
                    None,
                    clone!(@weak self as obj => move |chat, pspec| {
                        if pspec.name() == "is-pinned"
                            || pspec.name() == "is-marked-as-unread"
                            || pspec.name() == "unread-count"
                        {
                            obj.update_actions_for_chat(chat)
                        }
                    }),
                );
                self.imp().chat_notify_handler_id.replace(Some(handler_id));
        } else {
                self.update_pin_actions(false, false);
                self.update_mark_as_unread_actions(false, false);
        }
    }

    fn update_actions_for_chat(&self, chat: &Chat) {
        self.update_pin_actions(!chat.is_pinned(), chat.is_pinned());
        if chat.unread_count() > 0 {
            self.update_mark_as_unread_actions(false, true);
        } else {
            self.update_mark_as_unread_actions(
                !chat.is_marked_as_unread(),
                chat.is_marked_as_unread(),
            );
        }
    }

    fn update_pin_actions(&self, pin: bool, unpin: bool) {
        self.action_set_enabled("sidebar-row.pin", pin);
        self.action_set_enabled("sidebar-row.unpin", unpin);
    }

    fn update_mark_as_unread_actions(&self, unread: bool, read: bool) {
        self.action_set_enabled("sidebar-row.mark-as-unread", unread);
        self.action_set_enabled("sidebar-row.mark-as-read", read);
    }
}

fn sender_label(message: Message) -> Option<String> {
    use MessageContent::*;

    // Just use a sender label for specific messages.
    match message.content().0 {
        MessageText(_) | MessageSticker(_) | MessagePhoto(_) | MessageAudio(_)
        | MessageAnimation(_) | MessageVideo(_) | MessageDocument(_) | MessageVoiceNote(_)
        | MessageCall(_) => {}
        _ => return None,
    }

    if message.chat().is_own_chat() {
        if message.is_outgoing() {
            None
        } else {
            message
                .forward_info()
                .map(MessageForwardInfo::origin)
                .map(|forward_origin| match forward_origin {
                    MessageForwardOrigin::User(user) => strings::user_display_name(user, false),
                    MessageForwardOrigin::Chat { chat, .. }
                    | MessageForwardOrigin::Channel { chat, .. } => chat.title(),
                    MessageForwardOrigin::HiddenUser { sender_name }
                    | MessageForwardOrigin::MessageImport { sender_name } => sender_name.clone(),
                })
        }
    } else {
        let show_sender = match message.chat().type_() {
            ChatType::BasicGroup(_) => true,
            ChatType::Supergroup(supergroup) => !supergroup.is_channel(),
            ChatType::Private(_) | ChatType::Secret(_) => message.is_outgoing(),
        };
        if show_sender {
            Some(if message.is_outgoing() {
                gettext("You")
            } else {
                escape(&strings::message_sender(message.sender(), false))
            })
        } else {
            None
        }
    }
}

fn message_thumbnail_texture(message: Message) -> Option<gdk::Texture> {
    match message.content().0 {
        MessageContent::MessageAnimation(data) => data.animation.minithumbnail,
        MessageContent::MessageAudio(data) => data.audio.album_cover_minithumbnail,
        MessageContent::MessageChatChangePhoto(data) => data.photo.minithumbnail,
        MessageContent::MessageDocument(data) => data.document.minithumbnail,
        MessageContent::MessagePhoto(data) => data.photo.minithumbnail,
        MessageContent::MessageVideo(data) => data.video.minithumbnail,
        _ => None,
    }
    .map(|thumbnail| {
        gdk::Texture::from_bytes(&glib::Bytes::from_owned(glib::base64_decode(
            &thumbnail.data,
        )))
        .unwrap()
    })
}

fn draft_message_text(message: DraftMessage) -> String {
    match message.input_message_text {
        InputMessageContent::InputMessageText(data) => data.text.text,
        other => {
            log::warn!("Unexpected draft message type: {other:?}");
            String::new()
        }
    }
}

fn stringify_action(action: &ChatAction) -> String {
    use tdlib::enums::ChatAction::*;

    let show_sender = matches!(
        action.chat().type_(),
        ChatType::BasicGroup(_) | ChatType::Supergroup(_)
    );

    let td_action = &action.type_().0;

    let action_group = action.chat().actions().group(td_action);

    match td_action {
        ChoosingContact => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is choosing a contact",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are choosing contacts",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are choosing contacts", len),
                }
            } else {
                gettext("choosing a contact")
            }
        }
        ChoosingLocation => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is choosing a location",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are choosing locations",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are choosing locations", len),
                }
            } else {
                gettext("choosing a location")
            }
        }
        ChoosingSticker => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is choosing a sticker",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are choosing stickers",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are choosing stickers", len),
                }
            } else {
                gettext("choosing a sticker")
            }
        }
        RecordingVideo => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is recording a video",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are recording videos",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are recording videos", len),
                }
            } else {
                gettext("recording a video")
            }
        }
        RecordingVideoNote => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is recording a video note",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are recording video notes",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are recording video notes", len),
                }
            } else {
                gettext("recording a video note")
            }
        }
        RecordingVoiceNote => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is recording a voice note",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are recording voice notes",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are recording voice notes", len),
                }
            } else {
                gettext("recording a voice note")
            }
        }
        StartPlayingGame => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is playing a game",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are playing games",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are playing games", len),
                }
            } else {
                gettext("playing a game")
            }
        }
        Typing => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is typing",
                        strings::message_sender(action_group[0].sender(), false)
                    ),
                    2 => gettext!(
                        "{} and {} are typing",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are typing", len),
                }
            } else {
                gettext("typing")
            }
        }
        UploadingDocument(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is uploading a document ({}%)",
                        strings::message_sender(action_group[0].sender(), false),
                        action.progress,
                    ),
                    2 => gettext!(
                        "{} and {} are uploading documents",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are uploading documents", len),
                }
            } else {
                gettext!("uploading a document ({}%)", action.progress)
            }
        }
        UploadingPhoto(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is uploading a photo ({}%)",
                        strings::message_sender(action_group[0].sender(), false),
                        action.progress,
                    ),
                    2 => gettext!(
                        "{} and {} are uploading photos",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are uploading photos", len),
                }
            } else {
                gettext!("uploading a photo ({}%)", action.progress)
            }
        }
        UploadingVideo(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is uploading a video ({}%)",
                        strings::message_sender(action_group[0].sender(), false),
                        action.progress,
                    ),
                    2 => gettext!(
                        "{} and {} are uploading videos",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are uploading videos", len),
                }
            } else {
                gettext!("uploading a video ({}%)", action.progress)
            }
        }
        UploadingVideoNote(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is uploading a video note ({}%)",
                        strings::message_sender(action_group[0].sender(), false),
                        action.progress,
                    ),
                    2 => gettext!(
                        "{} and {} are uploading video notes",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are uploading video notes", len),
                }
            } else {
                gettext!("uploading a video note ({}%)", action.progress)
            }
        }
        UploadingVoiceNote(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is uploading a voice note ({}%)",
                        strings::message_sender(action_group[0].sender(), false),
                        action.progress,
                    ),
                    2 => gettext!(
                        "{} and {} are uploading voice notes",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                    ),
                    len => gettext!("{} people are uploading voice notes", len),
                }
            } else {
                gettext!("uploading a voice note ({}%)", action.progress)
            }
        }
        WatchingAnimations(action) => {
            if show_sender {
                match action_group.len() {
                    1 => gettext!(
                        "{} is watching an animation {}",
                        strings::message_sender(action_group[0].sender(), false),
                        action.emoji
                    ),
                    2 => gettext!(
                        "{} and {} are watching animations {}",
                        strings::message_sender(action_group[0].sender(), false),
                        strings::message_sender(action_group[1].sender(), false),
                        action.emoji
                    ),
                    len => gettext!("{} people are watching animations {}", len, action.emoji),
                }
            } else {
                gettext("watching an animation")
            }
        }
        Cancel => unreachable!(),
    }
}
