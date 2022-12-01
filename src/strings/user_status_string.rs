use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::enums::UserStatus;

use crate::strings;
use crate::tdlib::User;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;

    #[derive(Debug, Default)]
    pub(crate) struct UserStatusString(pub(super) OnceCell<User>);

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

        user.connect_notify_local(
            Some("status"),
            clone!(@weak obj => move |_, _| {
                obj.notify("string");
            }),
        );

        // Notify the string every minute when the user is offline so that
        // the "last seen" text is updated when time passes.
        glib::timeout_add_seconds_local(
            60,
            clone!(@weak obj => @default-return glib::Continue(false), move || {
                let user = obj.imp().0.get().unwrap();
                if let UserStatus::Offline(_) = user.status().0 {
                    obj.notify("string");
                }
                glib::Continue(true)
            }),
        );

        obj.imp().0.set(user).unwrap();
        obj
    }

    pub(crate) fn string(&self) -> String {
        let user = self.imp().0.get().unwrap();
        strings::user_status(&user.status().0)
    }
}
