use std::cell::OnceCell;
use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;

use crate::session::sidebar::search::ItemRow;
use crate::session::sidebar::search::Section;
use crate::session::sidebar::search::SectionRow;
use crate::session::sidebar::search::SectionType;
use crate::tdlib::Chat;
use crate::tdlib::User;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Row {
        /// A `Chat` or `User`
        pub(super) item: RefCell<Option<glib::Object>>,
        pub(super) list_item: OnceCell<gtk::ListItem>,
        pub(super) child: RefCell<Option<gtk::Widget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "SidebarSearchRow";
        type Type = super::Row;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<glib::Object>("item")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<gtk::ListItem>("list-item")
                        .write_only()
                        .construct_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.set_item(value.get().unwrap()),
                "list-item" => self.list_item.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "item" => obj.item().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            if let Some(child) = self.child.take() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Row {}
}

glib::wrapper! {
    pub(crate) struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}

impl Row {
    pub(crate) fn set_item(&self, item: Option<glib::Object>) {
        if self.item() == item {
            return;
        }

        let imp = self.imp();

        if item
            .as_ref()
            .map(|i| i.type_() == Chat::static_type() || i.type_() == User::static_type())
            .unwrap_or_default()
        {
            imp.list_item.get().unwrap().set_activatable(true);
            self.update_or_create_item_row(item.clone());
        } else if let Some(section) = item.as_ref().and_then(|i| i.downcast_ref::<Section>()) {
            imp.list_item.get().unwrap().set_activatable(false);
            self.update_or_create_section_row(section.section_type());
        } else {
            if let Some(child) = imp.child.take() {
                child.unparent();
            }

            if let Some(ref item) = item {
                log::warn!("Unexpected item type: {:?}", item);
            }
        }

        imp.item.replace(item);
        self.notify("item");
    }

    pub(crate) fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().clone()
    }

    fn update_or_create_item_row(&self, item: Option<glib::Object>) {
        let mut child_ref = self.imp().child.borrow_mut();
        match child_ref.as_ref().and_then(|c| c.downcast_ref::<ItemRow>()) {
            Some(item_row) => {
                item_row.set_item(item);
            }
            None => {
                let item_row = ItemRow::new(&item);
                item_row.set_parent(self);
                *child_ref = Some(item_row.upcast());
            }
        }
    }

    fn update_or_create_section_row(&self, section_type: SectionType) {
        let mut child_ref = self.imp().child.borrow_mut();
        match child_ref
            .as_ref()
            .and_then(|c| c.downcast_ref::<SectionRow>())
        {
            Some(section_row) => {
                section_row.set_section_type(section_type);
            }
            None => {
                let section_row = SectionRow::new(section_type);
                section_row.set_parent(self);
                *child_ref = Some(section_row.upcast());
            }
        }
    }
}
