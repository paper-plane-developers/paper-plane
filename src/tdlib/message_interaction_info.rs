use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::types;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub(crate) struct MessageInteractionInfo {
        pub(super) reply_count: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageInteractionInfo {
        const NAME: &'static str = "MessageInteractionInfo";
        type Type = super::MessageInteractionInfo;
    }

    impl ObjectImpl for MessageInteractionInfo {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecUInt::builder("reply-count")
                    .read_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "reply-count" => self.obj().reply_count().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageInteractionInfo(ObjectSubclass<imp::MessageInteractionInfo>);
}

impl From<Option<types::MessageInteractionInfo>> for MessageInteractionInfo {
    fn from(interaction_info: Option<types::MessageInteractionInfo>) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp()
            .reply_count
            .set(extract_reply_count(interaction_info));
        obj
    }
}

impl MessageInteractionInfo {
    pub(crate) fn update(&self, interaction_info: Option<types::MessageInteractionInfo>) {
        self.set_reply_count(extract_reply_count(interaction_info));
    }

    pub(crate) fn reply_count(&self) -> u32 {
        self.imp().reply_count.get()
    }

    fn set_reply_count(&self, reply_count: u32) {
        if self.reply_count() == reply_count {
            return;
        }
        self.imp().reply_count.set(reply_count);
        self.notify("reply-count");
    }
}

fn extract_reply_count(interaction_info: Option<types::MessageInteractionInfo>) -> u32 {
    interaction_info
        .and_then(|interaction_info| interaction_info.reply_info)
        .map(|reply_info| reply_info.reply_count)
        .unwrap_or(0) as u32
}
