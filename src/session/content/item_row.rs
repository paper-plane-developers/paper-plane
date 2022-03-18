use adw::prelude::BinExt;
use adw::subclass::prelude::BinImpl;
use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::borrow::Cow;
use tdgrand::enums::{MessageContent, StickerType, UserType};

use crate::session::chat::{ChatType, Item, ItemType, Message, MessageSender, SponsoredMessage};
use crate::session::content::message_row::{MessagePhoto, MessageSticker, MessageText};
use crate::session::content::{EventRow, MessageRow, MessageRowExt};
use crate::session::User;
use crate::utils::MESSAGE_TRUNCATED_LENGTH;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct ItemRow {
        /// An `Item` or `SponsoredMessage`
        pub item: RefCell<Option<glib::Object>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemRow {
        const NAME: &'static str = "ContentItemRow";
        type Type = super::ItemRow;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for ItemRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "item",
                    "Item",
                    "The item represented by this row",
                    glib::Object::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                "item" => obj.set_item(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ItemRow {}
    impl BinImpl for ItemRow {}
}

glib::wrapper! {
    pub struct ItemRow(ObjectSubclass<imp::ItemRow>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for ItemRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ItemRow")
    }

    pub fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().to_owned()
    }

    pub fn set_item(&self, item: Option<glib::Object>) {
        if self.item() == item {
            return;
        }

        if let Some(ref item) = item {
            if let Some(item) = item.downcast_ref::<Item>() {
                match item.type_() {
                    ItemType::Message(message) => {
                        let content = message.content().0;

                        match content {
                            MessageContent::MessagePhoto(_) => {
                                self.set_child_row::<MessagePhoto>(message.to_owned().upcast())
                            }
                            MessageContent::MessageSticker(data)
                                if matches!(data.sticker.r#type, StickerType::Static)
                                    || matches!(data.sticker.r#type, StickerType::Mask(_)) =>
                            {
                                self.set_child_row::<MessageSticker>(message.to_owned().upcast())
                            }
                            MessageContent::MessageChatChangeTitle(data) => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    match message.chat().type_() {
                                        ChatType::Supergroup(supergroup)
                                            if supergroup.is_channel() =>
                                        {
                                            gettext!("Channel name was changed to «{}»", data.title)
                                        }
                                        _ => {
                                            if message.is_outgoing() {
                                                gettext!(
                                                    "You changed group name to «{}»",
                                                    data.title
                                                )
                                            } else {
                                                gettext!(
                                                    "{} changed group name to «{}»",
                                                    sender_display_name(message),
                                                    data.title
                                                )
                                            }
                                        }
                                    }
                                ));
                            }
                            //TODO display photo miniature next to/under "changed photo" label
                            MessageContent::MessageChatChangePhoto(_) => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    match message.chat().type_() {
                                        ChatType::Supergroup(data) if data.is_channel() => {
                                            gettext("Channel photo changed")
                                        }
                                        _ => {
                                            if message.is_outgoing() {
                                                gettext("You changed group photo")
                                            } else {
                                                gettext!(
                                                    "{} changed group photo",
                                                    sender_display_name(message)
                                                )
                                            }
                                        }
                                    }
                                ));
                            }
                            MessageContent::MessageChatDeletePhoto => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    match message.chat().type_() {
                                        ChatType::Supergroup(data) if data.is_channel() => {
                                            gettext("Channel photo removed")
                                        }
                                        _ => {
                                            if message.is_outgoing() {
                                                gettext("You removed group photo")
                                            } else {
                                                gettext!(
                                                    "{} removed group photo",
                                                    sender_display_name(message)
                                                )
                                            }
                                        }
                                    }
                                ));
                            }
                            MessageContent::MessageChatJoinByRequest => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    if message.is_outgoing() {
                                        gettext("You joined the group")
                                    } else {
                                        gettext!(
                                            "{} joined the group",
                                            sender_display_name(message)
                                        )
                                    }
                                ));
                            }
                            MessageContent::MessageChatJoinByLink => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    if message.is_outgoing() {
                                        gettext("You joined the group via invite link")
                                    } else {
                                        gettext!(
                                            "{} joined the group via invite link",
                                            sender_display_name(message)
                                        )
                                    }
                                ));
                            }
                            MessageContent::MessageChatAddMembers(data) => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    stringify_added_members(message, data.member_user_ids)
                                ));
                            }
                            MessageContent::MessageChatDeleteMember(data) => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    match message.sender() {
                                        MessageSender::User(user) if user.id() == data.user_id => {
                                            if message.is_outgoing() {
                                                gettext("You left the group")
                                            } else {
                                                gettext!(
                                                    "{} left the group",
                                                    sender_display_name(message)
                                                )
                                            }
                                        }
                                        _ => {
                                            let user_name = self::sender_name(
                                                &message
                                                    .chat()
                                                    .session()
                                                    .user_list()
                                                    .get(data.user_id),
                                            );
                                            if message.is_outgoing() {
                                                gettext!("You removed {} from the group", user_name)
                                            } else {
                                                gettext!(
                                                    "{} removed {} from the group",
                                                    sender_display_name(message),
                                                    user_name
                                                )
                                            }
                                        }
                                    }
                                ));
                            }
                            MessageContent::MessagePinMessage(data) => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    if message.is_outgoing() {
                                        // Translators: You pinned {a message}
                                        gettext!(
                                            "You pinned {}",
                                            stringify_pinned_message_content(
                                                message
                                                    .chat()
                                                    .history()
                                                    .message_by_id(data.message_id)
                                            )
                                        )
                                    } else {
                                        // Translators: {User} pinned {a message}
                                        gettext!(
                                            "{} pinned {}",
                                            sender_display_name(message),
                                            stringify_pinned_message_content(
                                                message
                                                    .chat()
                                                    .history()
                                                    .message_by_id(data.message_id)
                                            )
                                        )
                                    }
                                ));
                            }
                            MessageContent::MessageSupergroupChatCreate(data) => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    match message.chat().type_() {
                                        ChatType::Supergroup(supergroup)
                                            if supergroup.is_channel() =>
                                        {
                                            gettext!("Created the channel «{}»", data.title)
                                        }
                                        _ => {
                                            if message.is_outgoing() {
                                                gettext!("You created the group «{}»", data.title)
                                            } else {
                                                // Translators: {User} created the group «{group name}»
                                                gettext!(
                                                    "{} created the group «{}»",
                                                    sender_display_name(message),
                                                    data.title
                                                )
                                            }
                                        }
                                    }
                                ));
                            }
                            MessageContent::MessageBasicGroupChatCreate(data) => {
                                self.get_or_create_event_row().set_label(&format!(
                                    "<b>{}</b>",
                                    if message.is_outgoing() {
                                        gettext!("You created the group «{}»", data.title)
                                    } else {
                                        gettext!(
                                            "{} created the group «{}»",
                                            sender_display_name(message),
                                            data.title
                                        )
                                    }
                                ));
                            }
                            _ => self.set_child_row::<MessageText>(message.to_owned().upcast()),
                        }
                    }
                    ItemType::DayDivider(date) => {
                        let fmt = if date.year() == glib::DateTime::now_local().unwrap().year() {
                            // Translators: This is a date format in the day divider without the year
                            gettext("%B %e")
                        } else {
                            // Translators: This is a date format in the day divider with the year
                            gettext("%B %e, %Y")
                        };
                        let date = date.format(&format!("<b>{}</b>", fmt)).unwrap().to_string();

                        let child = self.get_or_create_event_row();
                        child.set_label(&date);
                    }
                }
            } else if let Some(sponsored_message) = item.downcast_ref::<SponsoredMessage>() {
                let content = &sponsored_message.content().0;
                if !matches!(content, MessageContent::MessageText(_)) {
                    log::warn!("Unexpected sponsored message of type: {:?}", content);
                }

                self.set_child_row::<MessageText>(sponsored_message.to_owned().upcast());
            } else {
                unreachable!("Unexpected item type: {:?}", item);
            }
        }

        self.imp().item.replace(item);
        self.notify("item");
    }

    fn set_child_row<M: IsA<gtk::Widget> + IsA<MessageRow> + MessageRowExt>(
        &self,
        message: glib::Object,
    ) {
        match self.child().and_then(|w| w.downcast::<M>().ok()) {
            Some(child) => child.set_message(Some(message)),
            None => {
                let child = M::new(&message);
                self.set_child(Some(&child));
            }
        }
    }

    fn get_or_create_event_row(&self) -> EventRow {
        if let Some(Ok(child)) = self.child().map(|w| w.downcast::<EventRow>()) {
            child
        } else {
            let child = EventRow::new();
            self.set_child(Some(&child));
            child
        }
    }
}

pub fn sender_name(user: &User) -> String {
    let type_ = user.type_().0;
    if type_ == UserType::Deleted {
        gettext("Deleted Account")
    } else {
        format!("{} {}", user.first_name(), user.last_name())
    }
}

fn sender_display_name(message: &Message) -> String {
    match message.sender() {
        MessageSender::User(data) => sender_name(data),
        MessageSender::Chat(data) => data.title(),
    }
}
fn stringify_pinned_message_content(message: Option<Message>) -> String {
    match message {
        Some(data) => match data.content().0 {
            MessageContent::MessageText(data) => {
                let msg = data.text.text;
                if msg.chars().count() > MESSAGE_TRUNCATED_LENGTH {
                    gettext!(
                        "«{}…»",
                        msg.chars()
                            .take(MESSAGE_TRUNCATED_LENGTH - 1)
                            .collect::<String>()
                    )
                } else {
                    gettext!("«{}»", msg)
                }
            }
            MessageContent::MessagePhoto(_) => gettext("a photo"),
            MessageContent::MessageVideo(_) => gettext("a video"),
            MessageContent::MessageSticker(data) => {
                gettext!("a {} sticker", data.sticker.emoji)
            }
            MessageContent::MessageAnimation(_) => gettext("a GIF"),
            MessageContent::MessageDocument(_) => gettext("a file"),
            MessageContent::MessageAudio(_) => gettext("an audio file"),
            _ => gettext("a message"),
        },
        None => gettext("a deleted message"),
    }
}

fn stringify_added_members(message: &Message, member_user_ids: Vec<i64>) -> String {
    let my_user_id = message.chat().session().me().id();
    if message.sender().as_user().map(User::id).as_ref() == member_user_ids.get(0) {
        if message.is_outgoing() {
            gettext("You joined the group")
        } else {
            gettext!("{} joined the group", sender_display_name(message))
        }
    } else {
        let session = message.chat().session();
        let user_list = session.user_list();
        let members = member_user_ids
            .iter()
            .copied()
            .filter(|user_id| *user_id != my_user_id)
            .map(|user_id| user_list.get(user_id))
            .map(|user| self::sender_name(&user))
            .collect::<Vec<_>>();
        if members.is_empty() {
            // Translators: User added you to the group
            gettext!("{} added you to the group", sender_display_name(message))
        } else {
            let (last_member, first_members) = members.split_last().unwrap();
            if message.is_outgoing() {
                gettext!(
                    "You added {} to the group",
                    if first_members.is_empty() {
                        Cow::Borrowed(last_member)
                    } else {
                        // Translators: This string is used to separate names of two users e.g. Tom and Jerry
                        Cow::Owned(gettext!(
                            "{} and {}",
                            // Translators: This comma is used to separate names of two users
                            first_members.join(&gettext(", ")),
                            last_member
                        ))
                    }
                )
            } else {
                gettext!(
                    "{} added {} to the group",
                    sender_display_name(message),
                    if first_members.is_empty() {
                        Cow::Borrowed(last_member)
                    } else {
                        Cow::Owned(if member_user_ids.len() != members.len() {
                            // Translators: This string is used to separate names of two users e.g. Tom and Jerry
                            gettext!(
                                "{} and you",
                                // Translators: This comma is used to separate names of two users
                                members.join(&gettext(", ")),
                            )
                        } else {
                            // Translators: This string is used to separate names of two users e.g. Tom and Jerry
                            gettext!(
                                "{} and {}",
                                // Translators: This comma is used to separate names of two users
                                first_members.join(&gettext(", ")),
                                last_member
                            )
                        })
                    }
                )
            }
        }
    }
}
