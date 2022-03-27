use gtk::glib;
use tdlib::types::{ChatPhotoInfo as TdChatPhotoInfo, File, ProfilePhoto as TdProfilePhoto};

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "Avatar", nullable)]
pub(crate) struct Avatar(pub(super) File);

impl From<TdChatPhotoInfo> for Avatar {
    fn from(td_chat_photo_info: TdChatPhotoInfo) -> Self {
        Self(td_chat_photo_info.small)
    }
}

impl From<TdProfilePhoto> for Avatar {
    fn from(td_profile_photo: TdProfilePhoto) -> Self {
        Self(td_profile_photo.small)
    }
}

impl PartialEq for Avatar {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
