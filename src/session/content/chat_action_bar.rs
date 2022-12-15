use anyhow::anyhow;
use gettextrs::gettext;
use glib::{clone, closure};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use tdlib::enums::{ChatAction, ChatMemberStatus, InputMessageContent, UserType};
use tdlib::{functions, types};

use crate::session::components::MessageEntry;
use crate::session::content::SendPhotoDialog;
use crate::tdlib::{
    BasicGroup, BoxedChatMemberStatus, BoxedChatPermissions, BoxedDraftMessage, BoxedFormattedText,
    BoxedUserType, Chat, ChatType, Supergroup, User,
};
use crate::utils::{spawn, temp_dir};
use crate::{expressions, strings};

const PHOTO_MIME_TYPES: &[&str] = &["image/png", "image/jpeg"];

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/content-chat-action-bar.ui")]
    pub(crate) struct ChatActionBar {
        pub(super) chat: RefCell<Option<Chat>>,
        pub(super) chat_action_in_cooldown: Cell<bool>,
        pub(super) reply_to_message_id: Cell<i64>,
        pub(super) emoji_chooser: RefCell<Option<gtk::EmojiChooser>>,
        pub(super) bindings: RefCell<Vec<gtk::ExpressionWatch>>,
        #[template_child]
        pub(super) top_bar_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub(super) top_bar_sender_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) top_bar_message_label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) message_entry: TemplateChild<MessageEntry>,
        #[template_child]
        pub(super) send_message_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) select_file_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) restriction_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatActionBar {
        const NAME: &'static str = "ContentChatActionBar";
        type Type = super::ChatActionBar;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();

            klass.install_action(
                "chat-action-bar.cancel-action",
                None,
                move |widget, _, _| {
                    widget.set_reply_to_message_id(0);
                },
            );
            klass.install_action("chat-action-bar.select-file", None, move |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    widget.select_file().await;
                }));
            });
            klass.install_action(
                "chat-action-bar.send-text-message",
                None,
                move |widget, _, _| {
                    spawn(clone!(@weak widget => async move {
                        widget.send_text_message().await;
                    }));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChatActionBar {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<Chat>("chat")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecInt64::builder("reply-to-message-id")
                        .explicit_notify()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "chat" => obj.set_chat(value.get().unwrap()),
                "reply-to-message-id" => obj.set_reply_to_message_id(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "chat" => obj.chat().to_value(),
                "reply-to-message-id" => obj.reply_to_message_id().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.layout_manager()
                .and_then(|l| l.downcast::<gtk::BoxLayout>().ok())
                .unwrap()
                .set_orientation(gtk::Orientation::Vertical);

            self.message_entry.connect_formatted_text_notify(
                clone!(@weak obj => move |message_entry, _| {
                    // Enable the send-text-message action only when the message entry contains
                    // at least one non-whitespace character
                    let should_enable = message_entry
                        .formatted_text()
                        .map(|f| f.0.text.contains(|c: char| !c.is_whitespace()))
                        .unwrap_or_default();
                    obj.action_set_enabled("chat-action-bar.send-text-message", should_enable);

                    // Send typing action
                    spawn(clone!(@weak obj => async move {
                        obj.send_chat_action(ChatAction::Typing).await;
                    }));
                }),
            );

            self.message_entry
                .connect_paste_clipboard(clone!(@weak obj => move |_| {
                    obj.handle_paste_action();
                }));

            self.message_entry
                .connect_emoji_button_press(clone!(@weak obj => move |_, button| {
                    obj.show_emoji_chooser(&button);
                }));

            // The message entry is always empty at this point, so disable the
            // send-text-message action
            obj.action_set_enabled("chat-action-bar.send-text-message", false);

            self.message_entry
                .connect_activate(clone!(@weak obj => move |_| {
                    obj.activate_action("chat-action-bar.send-text-message", None).unwrap()
                }));
        }

        fn dispose(&self) {
            self.message_entry.unparent();
            self.send_message_button.unparent();
            if let Some(emoji_chooser) = self.emoji_chooser.take() {
                emoji_chooser.unparent();
            }
        }
    }

    impl WidgetImpl for ChatActionBar {}
}

glib::wrapper! {
    pub(crate) struct ChatActionBar(ObjectSubclass<imp::ChatActionBar>)
        @extends gtk::Widget;
}

impl Default for ChatActionBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatActionBar {
    pub(crate) fn new() -> Self {
        glib::Object::builder().build()
    }

    fn update_top_bar(&self) {
        let imp = self.imp();
        let reply_to_message_id = imp.reply_to_message_id.get();

        if reply_to_message_id == 0 {
            imp.top_bar_sender_label.set_text(None);
            imp.top_bar_message_label.set_text(None);
            imp.top_bar_revealer.set_reveal_child(false);
        } else {
            if let Some(message) = self
                .chat()
                .and_then(|c| c.history().message_by_id(reply_to_message_id))
            {
                // TODO: Make these labels auto update
                imp.top_bar_sender_label
                    .set_text(Some(&strings::message_sender(message.sender())));
                imp.top_bar_message_label
                    .set_text(Some(&strings::message_content(&message)));
            } else {
                // TODO: Actually try using TDLib to retrieve the message before showing this
                imp.top_bar_sender_label.set_text(Some(&gettext("Unknown")));
                imp.top_bar_message_label
                    .set_text(Some(&gettext("Deleted Message")));
            }

            imp.top_bar_revealer.set_reveal_child(true);
        }
    }

    fn reset(&self) {
        self.imp().message_entry.set_formatted_text(None);
        self.set_reply_to_message_id(0);
    }

    async fn compose_text_message(&self) -> Option<InputMessageContent> {
        if let Some(formatted_text) = self.imp().message_entry.as_markdown().await {
            let content = types::InputMessageText {
                text: formatted_text,
                disable_web_page_preview: false,
                clear_draft: true,
            };

            Some(InputMessageContent::InputMessageText(content))
        } else {
            None
        }
    }

    fn show_emoji_chooser(&self, parent: &impl IsA<gtk::Widget>) {
        let imp = self.imp();
        let mut emoji_chooser = imp.emoji_chooser.borrow_mut();
        if emoji_chooser.is_none() {
            let chooser = gtk::EmojiChooser::new();
            chooser.set_parent(parent);
            chooser.connect_emoji_picked(clone!(@weak self as obj => move |_, emoji| {
                obj.imp().message_entry.insert_at_cursor(emoji);
            }));
            chooser.connect_hide(clone!(@weak self as obj => move |_| {
                obj.imp().message_entry.grab_focus();
            }));
            *emoji_chooser = Some(chooser);
        }
        emoji_chooser.as_ref().unwrap().popup();
    }

    async fn select_file(&self) {
        let parent_window = self.root().unwrap().downcast::<gtk::Window>().unwrap();
        let file_chooser = gtk::FileChooserNative::new(
            Some(&gettext("Open File")),
            Some(&parent_window),
            gtk::FileChooserAction::Open,
            Some(&gettext("_Open")),
            Some(&gettext("_Cancel")),
        );
        let filter = gtk::FileFilter::new();

        filter.set_name(Some(&gettext("Images")));
        for mime in PHOTO_MIME_TYPES {
            filter.add_mime_type(mime);
        }
        file_chooser.add_filter(&filter);

        if file_chooser.run_future().await == gtk::ResponseType::Accept {
            if let Some(file) = file_chooser.file() {
                let path = file.path().unwrap().to_str().unwrap().to_string();
                let chat = self.chat().unwrap();
                SendPhotoDialog::new(&Some(parent_window), chat, path).present();
            }
        }
    }

    async fn send_text_message(&self) {
        if let Some(chat) = self.chat() {
            if let Some(message) = self.compose_text_message().await {
                let client_id = chat.session().client_id();
                let chat_id = chat.id();
                let reply_to_message_id = self.imp().reply_to_message_id.get();

                // Send the message
                let result = functions::send_message(
                    chat_id,
                    0,
                    reply_to_message_id,
                    None,
                    message,
                    client_id,
                )
                .await;
                if let Err(e) = result {
                    log::warn!("Error sending a message: {:?}", e);
                }

                self.reset();
            }
        }
    }

    async fn save_message_as_draft(&self) {
        if let Some(chat) = self.chat() {
            let client_id = chat.session().client_id();
            let chat_id = chat.id();
            let reply_to_message_id = self.imp().reply_to_message_id.get();
            let draft_message =
                self.compose_text_message()
                    .await
                    .map(|message| types::DraftMessage {
                        reply_to_message_id,
                        date: glib::DateTime::now_local().unwrap().to_unix() as i32,
                        input_message_text: message,
                    });

            // Save draft message
            let result =
                functions::set_chat_draft_message(chat_id, 0, draft_message, client_id).await;
            if let Err(e) = result {
                log::warn!("Error setting a draft message: {:?}", e);
            }
        }
    }

    fn load_draft_message(&self, message: BoxedDraftMessage) {
        let imp = self.imp();

        if let InputMessageContent::InputMessageText(content) = message.0.input_message_text {
            imp.message_entry
                .set_formatted_text(Some(BoxedFormattedText(content.text)));
        } else {
            log::warn!(
                "Unexpected draft message type: {:?}",
                message.0.input_message_text
            );
            imp.message_entry.set_formatted_text(None);
        }

        self.set_reply_to_message_id(message.0.reply_to_message_id);
    }

    async fn send_chat_action(&self, action: ChatAction) {
        let imp = self.imp();
        if imp.chat_action_in_cooldown.get() {
            return;
        }

        if let Some(chat) = self.chat() {
            let client_id = chat.session().client_id();
            let chat_id = chat.id();

            // Enable chat action cooldown right away
            imp.chat_action_in_cooldown.set(true);

            // Send typing action
            let result = functions::send_chat_action(chat_id, 0, Some(action), client_id).await;
            if result.is_ok() {
                glib::timeout_add_seconds_local_once(
                    5,
                    clone!(@weak self as obj =>move || {
                        obj.imp().chat_action_in_cooldown.set(false);
                    }),
                );
            } else {
                imp.chat_action_in_cooldown.set(false);
            }
        }
    }

    pub(crate) fn handle_paste_action(&self) {
        if let Some(chat) = self.chat() {
            spawn(clone!(@weak self as obj => async move {
                if let Err(e) = obj.handle_image_clipboard(chat).await {
                    log::warn!("Error on pasting an image: {:?}", e);
                }
            }));
        }
    }

    async fn handle_image_clipboard(&self, chat: Chat) -> Result<(), anyhow::Error> {
        if let Ok((stream, mime)) = self
            .clipboard()
            .read_future(PHOTO_MIME_TYPES, glib::PRIORITY_DEFAULT)
            .await
        {
            let extension = match mime.as_str() {
                "image/png" => "png",
                "image/jpg" => "jpg",
                _ => unreachable!(),
            };

            let temp_dir =
                temp_dir().ok_or_else(|| anyhow!("The temporary directory doesn't exist"))?;
            let path = temp_dir.join("clipboard").with_extension(extension);

            save_stream_to_file(stream, &path).await?;

            let parent_window = self.root().unwrap().downcast().ok();
            let path = path.to_str().unwrap().to_string();
            SendPhotoDialog::new(&parent_window, chat, path).present();
        }

        Ok(())
    }

    pub(crate) fn chat(&self) -> Option<Chat> {
        self.imp().chat.borrow().clone()
    }

    fn set_chat(&self, chat: Option<Chat>) {
        if self.chat() == chat {
            return;
        }

        spawn(clone!(@weak self as obj => async move {
            obj.save_message_as_draft().await;
        }));

        let imp = self.imp();
        let mut bindings = imp.bindings.borrow_mut();
        while let Some(binding) = bindings.pop() {
            binding.unwatch();
        }

        if let Some(ref chat) = chat {
            if let Some(draft_message) = chat.draft_message() {
                self.load_draft_message(draft_message);
            } else {
                self.reset();
            }

            imp.chat_action_in_cooldown.set(false);

            let permissions_expression = Chat::this_expression("permissions");
            let is_blocked_expression = Chat::this_expression("is-blocked");

            // Handle whether or not message bar should be shown
            let message_bar_visibility_expression = match chat.type_() {
                ChatType::Private(data) => {
                    let user_type_expression =
                        gtk::ConstantExpression::new(data).chain_property::<User>("type");
                    message_bar_visibility_in_private_chats(
                        is_blocked_expression,
                        user_type_expression,
                    )
                }
                ChatType::Secret(data) => {
                    let user_type_expression =
                        gtk::ConstantExpression::new(data.user()).chain_property::<User>("type");
                    message_bar_visibility_in_private_chats(
                        is_blocked_expression,
                        user_type_expression,
                    )
                }
                ChatType::Supergroup(data) => {
                    let user_status_expression =
                        gtk::ConstantExpression::new(data).chain_property::<Supergroup>("status");
                    if data.is_channel() {
                        gtk::ClosureExpression::with_callback(&[user_status_expression], |args| {
                            let status = args[1].get::<BoxedChatMemberStatus>().unwrap().0;
                            matches!(
                                status,
                                ChatMemberStatus::Creator(_) | ChatMemberStatus::Administrator(_)
                            )
                        })
                        .upcast()
                    } else {
                        message_bar_visibility_in_groups(
                            permissions_expression.clone(),
                            user_status_expression,
                        )
                    }
                }
                ChatType::BasicGroup(data) => {
                    let user_status_expression =
                        gtk::ConstantExpression::new(data).chain_property::<BasicGroup>("status");
                    message_bar_visibility_in_groups(
                        permissions_expression.clone(),
                        user_status_expression,
                    )
                }
            };

            let message_bar_visibility_binding =
                message_bar_visibility_expression.bind(&*imp.message_entry, "visible", Some(chat));
            let send_button_visibility_binding = message_bar_visibility_expression.bind(
                &*imp.send_message_button,
                "visible",
                Some(chat),
            );
            // TODO: So in order to implement it correctly we need to duplicate message_bar_visibility_expression
            // to only change 3 LOC, so there must be a more efficient way of solving that issue
            // But for now I'm just leaving it like that, it's still better than nothing
            let select_file_visibility_binding = message_bar_visibility_expression.bind(
                &*imp.select_file_button,
                "visible",
                Some(chat),
            );
            bindings.push(message_bar_visibility_binding);
            bindings.push(send_button_visibility_binding);
            bindings.push(select_file_visibility_binding);

            // Handle whether or not restriction label should be shown
            let restriction_label_visibility_binding = permissions_expression
                .chain_closure::<bool>(closure!(|chat: Chat, permissions: BoxedChatPermissions| {
                    if permissions.0.can_send_messages {
                        match chat.type_() {
                            ChatType::Supergroup(data) => Some(data.status().0),
                            ChatType::BasicGroup(data) => Some(data.status().0),
                            _ => None,
                        }
                        .map(|status| match status {
                            ChatMemberStatus::Restricted(status) => {
                                !status.permissions.can_send_messages
                            }
                            _ => false,
                        })
                        .unwrap_or(false)
                    } else {
                        match chat.type_() {
                            ChatType::Supergroup(data) if !data.is_channel() => !matches!(
                                data.status().0,
                                ChatMemberStatus::Creator(_) | ChatMemberStatus::Administrator(_)
                            ),
                            ChatType::BasicGroup(data) => !matches!(
                                data.status().0,
                                ChatMemberStatus::Creator(_) | ChatMemberStatus::Administrator(_)
                            ),
                            _ => false,
                        }
                    }
                }))
                .upcast()
                .bind(&*imp.restriction_label, "visible", Some(chat));

            bindings.push(restriction_label_visibility_binding);

            // Handle restriction_label caption
            let restriction_label_binding = expressions::restriction_expression(chat).bind(
                &*imp.restriction_label,
                "label",
                Some(chat),
            );
            bindings.push(restriction_label_binding)
        }

        imp.chat.replace(chat);
        self.notify("chat");
    }

    pub(crate) fn reply_to_message_id(&self) -> i64 {
        self.imp().reply_to_message_id.get()
    }

    pub(crate) fn set_reply_to_message_id(&self, reply_to_message_id: i64) {
        if self.reply_to_message_id() == reply_to_message_id {
            return;
        }

        self.imp().reply_to_message_id.set(reply_to_message_id);
        self.update_top_bar();

        self.notify("reply-to-message-id");
    }
}

async fn save_stream_to_file(
    stream: gio::InputStream,
    path: impl AsRef<std::path::Path>,
) -> Result<(), glib::Error> {
    let file = gio::File::for_path(path);
    let file_stream = file
        .replace_future(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            glib::PRIORITY_DEFAULT,
        )
        .await?;

    file_stream
        .splice_future(
            &stream,
            gio::OutputStreamSpliceFlags::CLOSE_SOURCE | gio::OutputStreamSpliceFlags::CLOSE_TARGET,
            glib::PRIORITY_DEFAULT,
        )
        .await?;

    Ok(())
}

fn message_bar_visibility_in_private_chats(
    is_blocked_expression: gtk::PropertyExpression,
    user_type_expression: gtk::PropertyExpression,
) -> gtk::Expression {
    gtk::ClosureExpression::new::<bool>(
        &[is_blocked_expression, user_type_expression],
        closure!(|_: Chat, is_blocked: bool, user_type: BoxedUserType| {
            // Hide message bar if account is deleted
            if let UserType::Deleted = user_type.0 {
                false
            } else {
                !is_blocked
            }
        }),
    )
    .upcast()
}

fn message_bar_visibility_in_groups(
    permissions_expression: gtk::PropertyExpression,
    user_status_expression: gtk::PropertyExpression,
) -> gtk::Expression {
    gtk::ClosureExpression::new::<bool>(
        &[permissions_expression, user_status_expression],
        closure!(
            |_: Chat, permissions: BoxedChatPermissions, status: BoxedChatMemberStatus| {
                match status.0 {
                    ChatMemberStatus::Restricted(data) if !data.permissions.can_send_messages => {
                        false
                    }
                    ChatMemberStatus::Left | ChatMemberStatus::Banned(_) => false,
                    // Owner and admins are always allowed to send messages
                    ChatMemberStatus::Creator(_) | ChatMemberStatus::Administrator(_) => true,
                    _ => permissions.0.can_send_messages,
                }
            }
        ),
    )
    .upcast()
}
