use std::cell::OnceCell;
use std::cell::RefCell;

use gettextrs::gettext;
use glib::clone;
use glib::closure;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use tdlib::enums::InputMessageContent;
use tdlib::enums::MessageContent;
use tdlib::enums::MessageSendingState;
use tdlib::types::DraftMessage;

use crate::expressions;
use crate::session::sidebar::mini_thumbnail::MiniThumbnail;
use crate::session::sidebar::Avatar;
use crate::session::sidebar::Sidebar;
use crate::strings;
use crate::tdlib::Chat;
use crate::tdlib::ChatListItem;
use crate::tdlib::ChatType;
use crate::tdlib::Message;
use crate::tdlib::MessageForwardInfo;
use crate::tdlib::MessageForwardOrigin;
use crate::utils::spawn;
use crate::Session;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/sidebar-row.ui")]
    pub(crate) struct Row {
        pub(super) item: RefCell<Option<ChatListItem>>,
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
        pub(super) subtitle_label: TemplateChild<gtk::Inscription>,
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
        item_signal_group.connect_notify_local(
            Some("is-pinned"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_status_stack();
                obj.update_actions();
            }),
        );
        imp.item_signal_group.set(item_signal_group).unwrap();

        let chat_signal_group = glib::SignalGroup::new(Chat::static_type());
        chat_signal_group.connect_notify_local(
            Some("is-marked-as-unread"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_status_stack();
                obj.update_actions();
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("unread-count"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_status_stack();
                obj.update_actions();
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("unread-mention-count"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_status_stack();
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("last-read-outbox-message-id"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_message_status_icon();
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("last-message"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_message_status_icon();
                obj.update_timestamp();
                obj.update_subtitle_prefix_label();
                obj.update_minithumbnail();
                obj.update_subtitle_label();
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("draft-message"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_timestamp();
                obj.update_subtitle_prefix_label();
                obj.update_minithumbnail();
                obj.update_subtitle_label();
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("actions"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_subtitle_prefix_label();
                obj.update_minithumbnail();
                obj.update_subtitle_label();
            }),
        );
        chat_signal_group.connect_notify_local(
            Some("notification-settings"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_unread_count_style();
            }),
        );
        imp.chat_signal_group.set(chat_signal_group).unwrap();

        let session_signal_group = glib::SignalGroup::new(Session::static_type());
        session_signal_group.connect_notify_local(
            Some("private-chats-notification-settings"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_unread_count_style();
            }),
        );
        session_signal_group.connect_notify_local(
            Some("group-chats-notification-settings"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_unread_count_style();
            }),
        );
        session_signal_group.connect_notify_local(
            Some("channel-chats-notification-settings"),
            clone!(@weak self as obj => move |_, _| {
                obj.update_unread_count_style();
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
        self.update_timestamp();
        self.update_subtitle_prefix_label();
        self.update_minithumbnail();
        self.update_subtitle_label();
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
                        MessageSendingState::Pending(_) => {
                            ("message-pending-symbolic", "dim-label")
                        }
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

    fn update_timestamp(&self) {
        if let Some(item) = self.item() {
            let imp = self.imp();
            let chat = item.chat();
            let label = &imp.timestamp_label;

            let date = chat
                .draft_message()
                .map(|m| m.0.date)
                .or_else(|| chat.last_message().map(|m| m.date()));

            if let Some(date) = date {
                label.set_label(&timestamp_text(date as i64));
                label.set_visible(true);
            } else {
                label.set_visible(false);
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

    fn update_subtitle_label(&self) {
        if let Some(item) = self.item() {
            let imp = self.imp();
            let chat = item.chat();
            let label = &imp.subtitle_label;

            if let Some(actions) = chat.actions().last().map(|a| strings::chat_action(&a)) {
                label.set_text(Some(&actions));
                label.remove_css_class("dim-label");
                label.add_css_class("accent");
            } else if let Some(draft_message) =
                chat.draft_message().map(|d| draft_message_text(d.0))
            {
                label.set_text(Some(&draft_message));
                label.add_css_class("dim-label");
                label.remove_css_class("accent");
            } else if let Some(last_message) =
                chat.last_message().map(|m| strings::message_content(&m))
            {
                label.set_text(Some(&last_message));
                label.add_css_class("dim-label");
                label.remove_css_class("accent");
            } else {
                label.set_text(None);
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

fn timestamp_text(date: i64) -> glib::GString {
    let datetime_now = glib::DateTime::now_local().unwrap();
    let datetime = glib::DateTime::from_unix_utc(date)
        .and_then(|t| t.to_local())
        .unwrap();

    let difference = datetime_now.difference(&datetime);
    let hours_difference = difference.as_hours();
    let days_difference = difference.as_days();

    if hours_difference <= 16 {
        datetime.format(&gettext("%l:%M %p"))
    } else if days_difference < 6 {
        // Show the day of the week
        datetime.format("%a")
    } else if days_difference < 364 {
        // Show the day and the month
        datetime.format("%d %b")
    } else {
        // Show the entire date
        datetime.format("%x")
    }
    .unwrap()
}
