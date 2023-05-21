use std::cell::Cell;
use std::convert::TryFrom;

use gtk::gio::ListStore;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;

use super::add_account::AddAccountRow;
use super::session_entry_row::SessionEntryRow;
use crate::session::Session;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ExtraItemObj(pub(super) Cell<super::ExtraItem>);

    #[glib::object_subclass]
    impl ObjectSubclass for ExtraItemObj {
        const NAME: &'static str = "ExtraItemObj";
        type Type = super::ExtraItemObj;
    }

    impl ObjectImpl for ExtraItemObj {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecEnum::builder::<ExtraItem>("inner")
                    .construct_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "inner" => obj.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "inner" => self.0.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(u32)]
#[enum_type(name = "ExtraItem")]
pub(crate) enum ExtraItem {
    Separator = 0,
    AddAccount = 1,
}

impl ExtraItem {
    const VALUES: [Self; 2] = [Self::Separator, Self::AddAccount];
}

impl Default for ExtraItem {
    fn default() -> Self {
        Self::Separator
    }
}

glib::wrapper! {
    pub(crate) struct ExtraItemObj(ObjectSubclass<imp::ExtraItemObj>);
}

impl From<&ExtraItem> for ExtraItemObj {
    fn from(item: &ExtraItem) -> Self {
        glib::Object::builder().property("inner", item).build()
    }
}

impl ExtraItemObj {
    pub(crate) fn list_store() -> ListStore {
        ExtraItem::VALUES.iter().map(ExtraItemObj::from).fold(
            ListStore::new(ExtraItemObj::static_type()),
            |list_items, item| {
                list_items.append(&item);
                list_items
            },
        )
    }

    pub(crate) fn get(&self) -> ExtraItem {
        self.imp().0.get()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Session(Session, bool),
    Separator,
    AddAccount,
}

impl From<ExtraItem> for Item {
    fn from(extra_item: ExtraItem) -> Self {
        match extra_item {
            ExtraItem::Separator => Self::Separator,
            ExtraItem::AddAccount => Self::AddAccount,
        }
    }
}

impl TryFrom<glib::Object> for Item {
    type Error = glib::Object;

    fn try_from(object: glib::Object) -> Result<Self, Self::Error> {
        object
            .downcast::<gtk::StackPage>()
            .map(|sp| Self::Session(sp.child().downcast::<Session>().unwrap(), false))
            .or_else(|object| object.downcast::<ExtraItemObj>().map(|it| it.get().into()))
    }
}

impl Item {
    pub(crate) fn set_hint(self, this_session: Session) -> Self {
        match self {
            Self::Session(session, _) => {
                let hinted = this_session == session;
                Self::Session(session, hinted)
            }
            other => other,
        }
    }

    pub(crate) fn build_widget(&self) -> gtk::Widget {
        match self {
            Self::Session(ref session, hinted) => {
                let session_entry = SessionEntryRow::new(session);
                session_entry.set_hint(*hinted);
                session_entry.upcast()
            }
            Self::Separator => gtk::Separator::new(gtk::Orientation::Vertical).upcast(),
            Self::AddAccount => AddAccountRow::new().upcast(),
        }
    }
}
