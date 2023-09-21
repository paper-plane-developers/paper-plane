use gtk::glib;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "Avatar", nullable)]
pub(crate) struct Avatar(pub(crate) tdlib::types::File);

impl From<tdlib::types::ChatPhotoInfo> for Avatar {
    fn from(chat_photo_info: tdlib::types::ChatPhotoInfo) -> Self {
        Self(chat_photo_info.small)
    }
}

impl From<tdlib::types::ProfilePhoto> for Avatar {
    fn from(profile_photo: tdlib::types::ProfilePhoto) -> Self {
        Self(profile_photo.small)
    }
}

impl PartialEq for Avatar {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
