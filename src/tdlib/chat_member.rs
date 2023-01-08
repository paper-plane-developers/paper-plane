use super::User;
use gettextrs::gettext;
use glib::subclass::prelude::*;
use gtk::glib;
use tdlib::enums::ChatMemberStatus;
use tdlib::types::ChatMember as TdChatMember;

mod imp {
    use super::*;
    use glib::once_cell::sync::OnceCell;

    #[derive(Default)]
    pub(crate) struct ChatMember {
        pub(super) member: OnceCell<TdChatMember>,
        pub(super) user: OnceCell<User>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatMember {
        const NAME: &'static str = "ChatMember";
        type Type = super::ChatMember;
    }

    impl ObjectImpl for ChatMember {}
}

glib::wrapper! {
    pub(crate) struct ChatMember(ObjectSubclass<imp::ChatMember>);
}

impl ChatMember {
    pub fn new(member: TdChatMember, user: User) -> Self {
        let obj: Self = glib::Object::new(&[]);
        obj.imp().member.set(member).unwrap();
        obj.imp().user.set(user).unwrap();
        obj
    }

    pub fn status(&self) -> String {
        match self.imp().member.get().unwrap().status {
            ChatMemberStatus::Creator(ref owner) => {
                if owner.custom_title.is_empty() {
                    gettext("Owner")
                } else {
                    owner.custom_title.to_owned()
                }
            }
            ChatMemberStatus::Administrator(ref admin) => {
                if admin.custom_title.is_empty() {
                    gettext("Admin")
                } else {
                    admin.custom_title.to_owned()
                }
            }
            _ => "".to_string(),
        }
    }

    pub fn user(&self) -> User {
        self.imp().user.get().unwrap().clone()
    }
}
