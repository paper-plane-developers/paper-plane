use gettextrs::gettext;
use glib::clone;
use glib::source::SourceId;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::UserStatus;
use tdlib::enums::UserType;

use crate::strings;
use crate::tdlib::User;

mod imp {
    use std::cell::RefCell;

    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;

    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct UserStatusString {
        pub(super) user: OnceCell<User>,
        pub(super) source_id: RefCell<Option<SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserStatusString {
        const NAME: &'static str = "UserStatusString";
        type Type = super::UserStatusString;
    }

    impl ObjectImpl for UserStatusString {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecString::builder("string").read_only().build()]);
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "string" => obj.string().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct UserStatusString(ObjectSubclass<imp::UserStatusString>);
}

impl UserStatusString {
    pub(crate) fn new(user: User) -> UserStatusString {
        let obj: UserStatusString = glib::Object::builder().build();
        obj.imp().user.set(user).unwrap();
        let user = obj.imp().user.get().unwrap();

        user.connect_notify_local(
            Some("status"),
            clone!(@weak obj => move |_, _| {
                obj.handle_status_changed();
            }),
        );

        if let UserStatus::Offline(data) = user.status().0 {
            let last_online = glib::DateTime::from_unix_utc(data.was_online.into()).unwrap();
            let now_utc = glib::DateTime::now_utc().unwrap();
            let interval = (60 - (now_utc.second() - last_online.second())) % 60;
            glib::timeout_add_seconds_local_once(
                interval.try_into().unwrap_or_default(),
                clone!(@weak obj => move || {
                    let user = obj.imp().user.get().unwrap();
                    if let UserStatus::Offline(_) = user.status().0 {
                        obj.create_notify_loop();
                    }
                }),
            );
        } else {
            obj.handle_status_changed();
        };
        obj
    }

    fn create_notify_loop(&self) {
        // Notify the string every minute when the user is offline so that
        // the "last seen" text is updated when time passes.
        let source_id = glib::timeout_add_seconds_local(
            60,
            clone!(@weak self as obj => @default-return glib::Continue(false), move || {
                let user = obj.imp().user.get().unwrap();
                if let UserStatus::Offline(_) = user.status().0 {
                    obj.notify("string");
                }
                glib::Continue(true)
            }),
        );
        self.imp().source_id.replace(Some(source_id));
    }

    fn handle_status_changed(&self) {
        let user = self.imp().user.get().unwrap();
        self.notify("string");
        match user.status().0 {
            UserStatus::Offline(_) => self.create_notify_loop(),
            _ => {
                if let Some(source) = self.imp().source_id.take() {
                    source.remove();
                    self.imp().source_id.replace(None);
                }
            }
        }
    }

    pub(crate) fn string(&self) -> String {
        let user = self.imp().user.get().unwrap();
        if matches!(user.type_().0, UserType::Bot(_)) {
            gettext("bot")
        } else {
            strings::user_status(&user.status().0)
        }
    }
}
