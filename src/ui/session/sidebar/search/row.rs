use std::cell::OnceCell;
use std::cell::RefCell;
use std::sync::OnceLock;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::model;
use crate::ui;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct Row {
        /// A `Chat` or `User`
        pub(super) item: glib::WeakRef<glib::Object>,
        pub(super) list_item: OnceCell<gtk::ListItem>,
        pub(super) child: RefCell<Option<gtk::Widget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "PaplSidebarSearchRow";
        type Type = super::Row;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<glib::Object>("item")
                        .explicit_notify()
                        .build(),
                    glib::ParamSpecObject::builder::<gtk::ListItem>("list-item")
                        .write_only()
                        .construct_only()
                        .build(),
                ]
            })
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
    pub(crate) fn set_item(&self, item: Option<&glib::Object>) {
        if self.item().as_ref() == item {
            return;
        }

        let imp = self.imp();

        if item
            .as_ref()
            .map(|i| {
                i.type_() == model::Chat::static_type() || i.type_() == model::User::static_type()
            })
            .unwrap_or_default()
        {
            imp.list_item.get().unwrap().set_activatable(true);
            self.update_or_create_item_row(item);
        } else if let Some(section) = item
            .as_ref()
            .and_then(|i| i.downcast_ref::<ui::SidebarSearchSection>())
        {
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

        imp.item.set(item);
        self.notify("item");
    }

    pub(crate) fn item(&self) -> Option<glib::Object> {
        self.imp().item.upgrade()
    }

    fn update_or_create_item_row(&self, item: Option<&glib::Object>) {
        let mut child_ref = self.imp().child.borrow_mut();
        match child_ref
            .as_ref()
            .and_then(|c| c.downcast_ref::<ui::SidebarSearchItemRow>())
        {
            Some(item_row) => {
                item_row.set_item(item);
            }
            None => {
                let item_row = ui::SidebarSearchItemRow::new(item);
                item_row.set_parent(self);
                *child_ref = Some(item_row.upcast());
            }
        }
    }

    fn update_or_create_section_row(&self, section_type: ui::SidebarSearchSectionType) {
        let mut child_ref = self.imp().child.borrow_mut();
        match child_ref
            .as_ref()
            .and_then(|c| c.downcast_ref::<ui::SidebarSearchSectionRow>())
        {
            Some(section_row) => {
                section_row.set_section_type(section_type);
            }
            None => {
                let section_row = ui::SidebarSearchSectionRow::new(section_type);
                section_row.set_parent(self);
                *child_ref = Some(section_row.upcast());
            }
        }
    }
}
