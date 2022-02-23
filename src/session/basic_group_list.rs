use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
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

    /// Return the `BasicGroup` of the specified `id`. Panics if the basic group is not present.
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an `id` returned by TDLib, it should be expected that the
    /// relative `BasicGroup` exists in the list.
    pub fn get(&self, id: i64) -> BasicGroup {
        self.imp()
            .list
            .borrow()
            .get(&id)
            .expect("Failed to get expected BasicGroup")
            .to_owned()
    }

    pub fn handle_update(&self, update: &Update) {
        if let Update::BasicGroup(data) = update {
            let mut list = self.imp().list.borrow_mut();

            match list.entry(data.basic_group.id) {
                Entry::Occupied(entry) => entry.get().handle_update(update),
                Entry::Vacant(entry) => {
                    let basic_group = BasicGroup::from_td_object(&data.basic_group);
                    entry.insert(basic_group);

                    let position = (list.len() - 1) as u32;
                    drop(list);

                    self.items_changed(position, 0, 1);
                }
            }
        }
    }
}
