use gettextrs::gettext;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::UserStatusString;
use crate::i18n::*;
use crate::strings;
use crate::tdlib::Chat;
use crate::tdlib::ChatType;

mod imp {
    use std::cell::Cell;

    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;

    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ChatSubtitleString {
        pub(super) chat: OnceCell<Chat>,
        pub(super) show_actions: Cell<bool>,
        pub(super) user_status_string: OnceCell<UserStatusString>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatSubtitleString {
        const NAME: &'static str = "ChatSubtitleString";
        type Type = super::ChatSubtitleString;
    }

    impl ObjectImpl for ChatSubtitleString {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::builder("subtitle")
                    .read_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "subtitle" => obj.subtitle().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatSubtitleString(ObjectSubclass<imp::ChatSubtitleString>);
}

impl ChatSubtitleString {
    pub(crate) fn new(chat: Chat, should_show_actions: bool) -> ChatSubtitleString {
        let obj: ChatSubtitleString = glib::Object::builder().build();

        if should_show_actions {
            chat.actions()
                .connect_items_changed(clone!(@weak obj => move |_, _, _, _| {
                     obj.notify("subtitle");
                }));
        };

        chat.connect_notify_local(
            Some("online-member-count"),
            clone!(@weak obj => move |_, _| {
                obj.notify("subtitle");
            }),
        );
        match chat.type_() {
            ChatType::BasicGroup(basic) => {
                basic.connect_notify_local(
                    Some("member-count"),
                    clone!(@weak obj => move |_, _| {
                        obj.notify("subtitle");
                    }),
                );
            }
            ChatType::Supergroup(group) => {
                group.connect_notify_local(
                    Some("member-count"),
                    clone!(@weak obj => move |_, _| {
                        obj.notify("subtitle");
                    }),
                );
            }
            ChatType::Private(user) if !chat.is_own_chat() => {
                let user_status_string = UserStatusString::new(user.clone());
                user_status_string.connect_notify_local(
                    Some("string"),
                    clone!(@weak obj => move |_, _| {
                        obj.notify("subtitle");
                    }),
                );
                obj.imp()
                    .user_status_string
                    .set(user_status_string)
                    .unwrap();
            }
            ChatType::Secret(secret) => {
                let user = secret.user();
                let user_status_string = UserStatusString::new(user.clone());
                user_status_string.connect_notify_local(
                    Some("string"),
                    clone!(@weak obj => move |_, _| {
                        obj.notify("subtitle");
                    }),
                );
                obj.imp()
                    .user_status_string
                    .set(user_status_string)
                    .unwrap();
            }
            _ => (),
        }

        obj.imp().show_actions.set(should_show_actions);
        obj.imp().chat.set(chat).unwrap();
        obj
    }

    pub(crate) fn subtitle(&self) -> String {
        let imp = self.imp();
        let chat = imp.chat.get().unwrap();
        let should_show_actions = imp.show_actions.get();

        if let Some(action) = chat.actions().last() {
            should_show_actions
                .then(|| strings::chat_action(&action))
                .unwrap_or(String::new())
        } else {
            format!(
                "{}{}",
                if !chat.is_own_chat() {
                    match chat.type_() {
                        ChatType::Private(_) | ChatType::Secret(_) => {
                            if !chat.is_own_chat() {
                                imp.user_status_string.get().unwrap().string()
                            } else {
                                String::new()
                            }
                        }
                        ChatType::BasicGroup(basic) => {
                            let m_count = basic.member_count();
                            match m_count {
                                0 => gettext("group"),
                                _ => ngettext_f(
                                    "{num} member",
                                    "{num} members",
                                    m_count as u32,
                                    &[("num", &m_count.to_string())],
                                ),
                            }
                        }
                        ChatType::Supergroup(data) if data.is_channel() => {
                            let m_count = data.member_count();
                            match m_count {
                                0 => gettext("channel"),
                                _ => ngettext_f(
                                    "{num} subscriber",
                                    "{num} subscribers",
                                    m_count as u32,
                                    &[("num", &m_count.to_string())],
                                ),
                            }
                        }
                        ChatType::Supergroup(data) => {
                            let m_count = data.member_count();
                            match m_count {
                                0 => gettext("group"),
                                _ => ngettext_f(
                                    "{num} member",
                                    "{num} members",
                                    m_count as u32,
                                    &[("num", &m_count.to_string())],
                                ),
                            }
                        }
                    }
                } else {
                    String::new()
                },
                if chat.online_member_count() > 1 {
                    format!(
                        ", {}",
                        ngettext_f(
                            "{num} online",
                            "{num} online",
                            chat.online_member_count() as u32,
                            &[("num", &chat.online_member_count().to_string())]
                        )
                    )
                } else {
                    String::new()
                }
            )
        }
    }
}
