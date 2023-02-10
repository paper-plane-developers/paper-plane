use gettextrs::gettext;
use glib::{clone, closure};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};
use tdlib::enums::{InputMessageContent, MessageContent, MessageSendingState};
use tdlib::types::DraftMessage;

use crate::tdlib::{
    BoxedDraftMessage, BoxedMessageContent, Chat, ChatAction, ChatActionList, ChatListItem,
    ChatType, Message, MessageForwardInfo, MessageForwardOrigin,
};
use crate::utils::{dim_and_escape, spawn};
use crate::{expressions, strings, Session};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    use crate::session::sidebar::mini_thumbnail::MiniThumbnail;
    use crate::session::sidebar::{Avatar, Sidebar};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/sidebar-row.ui")]
    pub(crate) struct Row {
        pub(super) item: RefCell<Option<ChatListItem>>,
        pub(super) bindings: RefCell<Vec<gtk::ExpressionWatch>>,
        pub(super) item_signal_group: OnceCell<glib::SignalGroup>,
        pub(super) chat_signal_group: OnceCell<glib::SignalGroup>,
        pub(super) session_signal_group: OnceCell<glib::SignalGroup>,
        #[template_child]
        pub(super) title_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) message_status_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) timestamp_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) subtitle_prefix_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) minithumbnail: TemplateChild<MiniThumbnail>,
        #[template_child]
        pub(super) bottom_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) status_stack: TemplateChild<gtk::Stack>,
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
        fn on_pressed(&self, _n_press: i32, x: f64, y: f64) {
            self.show_menu(x as i32, y as i32);
        }

        #[template_callback]
        fn on_long_pressed(&self, x: f64, y: f64) {
            self.show_menu(x as i32, y as i32);
        }

        fn show_menu(&self, x: i32, y: i32) {
            let obj = self.obj();
            let sidebar = obj.ancestor(Sidebar::static_type()).unwrap();
            let menu = sidebar.downcast_ref::<Sidebar>().unwrap().row_menu();

            menu.set_pointing_to(Some(&gdk::Rectangle::new(x, y, 0, 0)));
            menu.unparent();
            menu.set_parent(&*obj);
            menu.popup();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<ChatListItem>("item")
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

            self.obj().setup_expressions();
            self.obj().create_signal_groups();
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
        glib::Object::new()
    }

    fn toggle_chat_is_pinned(&self) {
        if let Some(item) = self.item() {
            spawn(async move {
                if let Err(e) = item.toggle_is_pinned().await {
                    log::warn!("Error on toggling chat's pinned state: {e:?}");
                }
            });
        }
    }

    fn toggle_chat_marked_as_unread(&self) {
        if let Some(chat) = self.item().map(|i| i.chat()) {
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
        let chat_expression = item_expression.chain_property::<ChatListItem>("chat");

        // Chat title
        expressions::chat_display_name(&chat_expression).bind(
            &*imp.title_label,
            "text",
            Some(self),
        );

        // Chat unread count
        chat_expression
            .chain_property::<Chat>("unread-count")
            .chain_closure::<String>(closure!(|_: Self, unread_count: i32| {
                if unread_count > 0 {
                    unread_count.to_string()
                } else {
                    String::new()
                }
            }))
            .bind(&*imp.unread_count_label, "label", Some(self));
    }

    fn create_signal_groups(&self) {
        let imp = self.imp();

        let item_signal_group = glib::SignalGroup::new(ChatListItem::static_type());
        item_signal_group.connect_local(
            "notify::is-pinned",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_status_stack();
                obj.update_actions();
                None
            }),
        );
        imp.item_signal_group.set(item_signal_group).unwrap();

        let chat_signal_group = glib::SignalGroup::new(Chat::static_type());
        chat_signal_group.connect_local(
            "notify::is-marked-as-unread",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_status_stack();
                obj.update_actions();
                None
            }),
        );
        chat_signal_group.connect_local(
            "notify::unread-count",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_status_stack();
                obj.update_actions();
                None
            }),
        );
        chat_signal_group.connect_local(
            "notify::unread-mention-count",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_status_stack();
                None
            }),
        );
        chat_signal_group.connect_local(
            "notify::last-read-outbox-message-id",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_message_status_icon();
                None
            }),
        );
        chat_signal_group.connect_local(
            "notify::last-message",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_message_status_icon();
                obj.update_subtitle_prefix_label();
                obj.update_minithumbnail();
                None
            }),
        );
        chat_signal_group.connect_local(
            "notify::draft-message",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_subtitle_prefix_label();
                obj.update_minithumbnail();
                None
            }),
        );
        chat_signal_group.connect_local(
            "notify::actions",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_subtitle_prefix_label();
                obj.update_minithumbnail();
                None
            }),
        );
        chat_signal_group.connect_local(
            "notify::notification-settings",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_unread_count_style();
                None
            }),
        );
        imp.chat_signal_group.set(chat_signal_group).unwrap();

        let session_signal_group = glib::SignalGroup::new(Session::static_type());
        session_signal_group.connect_local(
            "notify::private-chats-notification-settings",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_unread_count_style();
                None
            }),
        );
        session_signal_group.connect_local(
            "notify::group-chats-notification-settings",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_unread_count_style();
                None
            }),
        );
        session_signal_group.connect_local(
            "notify::channel-chats-notification-settings",
            false,
            clone!(@weak self as obj => @default-return None, move |_| {
                obj.update_unread_count_style();
                None
            }),
        );
        imp.session_signal_group.set(session_signal_group).unwrap();
    }

    pub(crate) fn item(&self) -> Option<ChatListItem> {
        self.imp().item.borrow().clone()
    }

    pub(crate) fn set_item(&self, item: Option<ChatListItem>) {
        if self.item() == item {
            return;
        }

        let imp = self.imp();
        let mut bindings = imp.bindings.borrow_mut();

        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        if let Some(ref item) = item {
            let last_message_expression = Chat::this_expression("last-message");
            let draft_message_expression = Chat::this_expression("draft-message");
            let actions_expression = Chat::this_expression("actions");
            let chat = item.chat();

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
            .bind(&*imp.timestamp_label, "label", Some(&chat));
            bindings.push(timestamp_binding);

            let content_expression = last_message_expression.chain_property::<Message>("content");

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
                        .or_else(|| draft_message.map(|m| dim_and_escape(&draft_message_text(m.0))))
                        .or_else(|| {
                            last_message
                                .map(|message| dim_and_escape(&strings::message_content(&message)))
                        })
                        .unwrap_or_default()
                }),
            )
            .bind(&*imp.bottom_label, "markup", Some(&chat));
            bindings.push(message_binding);
        }

        imp.item_signal_group
            .get()
            .unwrap()
            .set_target(item.as_ref());
        imp.chat_signal_group
            .get()
            .unwrap()
            .set_target(item.as_ref().map(|i| i.chat()).as_ref());
        imp.session_signal_group
            .get()
            .unwrap()
            .set_target(item.as_ref().map(|i| i.chat().session()).as_ref());

        imp.item.replace(item);

        self.update_message_status_icon();
        self.update_subtitle_prefix_label();
        self.update_minithumbnail();
        self.update_status_stack();
        self.update_unread_count_style();
        self.update_actions();

        self.notify("item");
    }

    fn update_message_status_icon(&self) {
        if let Some(item) = self.item() {
            let imp = self.imp();
            let chat = item.chat();
            let icon = &imp.message_status_icon;

            if chat.is_own_chat() {
                icon.set_visible(false);
            } else if let Some(message) = chat.last_message().filter(Message::is_outgoing) {
                let (icon_name, css_class) = match message.sending_state() {
                    Some(state) => match state.0 {
                        MessageSendingState::Failed(_) => ("message-failed-symbolic", "error"),
                        MessageSendingState::Pending => ("message-pending-symbolic", "dim-label"),
                    },
                    None => (
                        if message.id() == chat.last_read_outbox_message_id() {
                            "message-read-symbolic"
                        } else {
                            "message-unread-right-symbolic"
                        },
                        "accent",
                    ),
                };

                icon.set_icon_name(Some(icon_name));
                icon.set_css_classes(&[css_class]);
                icon.set_visible(true);
            } else {
                icon.set_visible(false);
            }
        }
    }

    fn update_subtitle_prefix_label(&self) {
        if let Some(item) = self.item() {
            let imp = self.imp();
            let chat = item.chat();
            let label = &imp.subtitle_prefix_label;

            if chat.actions().last().is_some() {
                label.set_visible(false);
            } else if chat.draft_message().is_some() {
                label.set_label(&gettext("Draft:"));
                label.add_css_class("error");
                label.set_visible(true);
            } else if let Some(sender) = chat.last_message().and_then(sender_label) {
                label.set_label(&format!("{sender}:"));
                label.remove_css_class("error");
                label.set_visible(true);
            } else {
                label.set_visible(false);
            }
        }
    }

    fn update_minithumbnail(&self) {
        if let Some(item) = self.item() {
            let imp = self.imp();
            let chat = item.chat();
            let minithumbnail = &imp.minithumbnail;

            if chat.actions().n_items() > 0 || chat.draft_message().is_some() {
                minithumbnail.set_visible(false);
            } else if let Some(texture) = chat.last_message().and_then(message_thumbnail_texture) {
                minithumbnail.set_paintable(Some(texture.upcast()));
                minithumbnail.set_visible(true);
            } else {
                minithumbnail.set_visible(false);
            }
        }
    }

    fn update_status_stack(&self) {
        if let Some(item) = self.item() {
            let imp = self.imp();
            let chat = item.chat();
            let stack = &imp.status_stack;

            if chat.unread_mention_count() > 0 {
                stack.set_visible_child(&*imp.unread_mention_label);
                stack.set_visible(true);
            } else if chat.unread_count() > 0 || chat.is_marked_as_unread() {
                stack.set_visible_child(&*imp.unread_count_label);
                stack.set_visible(true);
            } else if item.is_pinned() {
                stack.set_visible_child(&*imp.pin_icon);
                stack.set_visible(true);
            } else {
                stack.set_visible(false);
            }
        }
    }

    fn update_unread_count_style(&self) {
        if let Some(item) = self.item() {
            let imp = self.imp();
            let chat = item.chat();
            let label = &imp.unread_count_label;

            let notification_settings = chat.notification_settings();
            let scope_notification_settings = match chat.type_() {
                ChatType::Private(_) | ChatType::Secret(_) => {
                    chat.session().private_chats_notification_settings()
                }
                ChatType::Supergroup(supergroup) if supergroup.is_channel() => {
                    chat.session().channel_chats_notification_settings()
                }
                _ => chat.session().group_chats_notification_settings(),
            };

            let css_class = if notification_settings.0.use_default_mute_for {
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
            };

            label.set_css_classes(&["unread-count", css_class]);
        }
    }

    fn update_actions(&self) {
        if let Some(item) = self.item() {
            let chat = item.chat();

            self.update_pin_actions(!item.is_pinned(), item.is_pinned());

            if chat.unread_count() > 0 {
                self.update_mark_as_unread_actions(false, true);
            } else {
                self.update_mark_as_unread_actions(
                    !chat.is_marked_as_unread(),
                    chat.is_marked_as_unread(),
                );
            }
        } else {
            self.update_pin_actions(false, false);
            self.update_mark_as_unread_actions(false, false);
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
                strings::message_sender(message.sender(), false)
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
