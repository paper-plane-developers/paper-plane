use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use indexmap::map::Entry;
use tdgrand::enums::Update;

use crate::session::BasicGroup;

mod imp {
    use super::*;
    use indexmap::IndexMap;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct BasicGroupList {
        pub list: RefCell<IndexMap<i64, BasicGroup>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BasicGroupList {
        const NAME: &'static str = "BasicGroupList";
        type Type = super::BasicGroupList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for BasicGroupList {}
    impl ListModelImpl for BasicGroupList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            BasicGroup::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|(_, i)| i.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct BasicGroupList(ObjectSubclass<imp::BasicGroupList>)
        @implements gio::ListModel;
}

impl Default for BasicGroupList {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicGroupList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create BasicGroupList")
    }

    pub fn get(&self, id: i64) -> Option<BasicGroup> {
        let self_ = imp::BasicGroupList::from_instance(self);
        self_.list.borrow().get(&id).cloned()
    }

    pub fn handle_update(&self, update: &Update) {
        if let Update::BasicGroup(data) = update {
            let self_ = imp::BasicGroupList::from_instance(self);
            let mut list = self_.list.borrow_mut();

            match list.entry(data.basic_group.id) {
                Entry::Occupied(entry) => entry.get().handle_update(update),
                Entry::Vacant(entry) => {
                    let basic_group = BasicGroup::from_td_object(&data.basic_group);
                    entry.insert(basic_group);

                    drop(list);

                    let position = (self_.list.borrow().len() - 1) as u32;
                    self.items_changed(position, 0, 1);
                }
            }
        }
    }
}
