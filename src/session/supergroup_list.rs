use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use indexmap::map::Entry;
use tdgrand::enums::Update;

use crate::session::Supergroup;

mod imp {
    use super::*;
    use indexmap::IndexMap;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct SupergroupList {
        pub list: RefCell<IndexMap<i64, Supergroup>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SupergroupList {
        const NAME: &'static str = "SupergroupList";
        type Type = super::SupergroupList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SupergroupList {}
    impl ListModelImpl for SupergroupList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Supergroup::static_type()
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
    pub struct SupergroupList(ObjectSubclass<imp::SupergroupList>)
        @implements gio::ListModel;
}

impl Default for SupergroupList {
    fn default() -> Self {
        Self::new()
    }
}

impl SupergroupList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SupergroupList")
    }

    /// Return the `Supergroup` of the specified `id`. Panics if the supergroup is not present.
    /// Note that TDLib guarantees that types are always returned before their ids,
    /// so if you use an `id` returned by TDLib, it should be expected that the
    /// relative `Supergroup` exists in the list.
    pub fn get(&self, id: i64) -> Supergroup {
        let self_ = imp::SupergroupList::from_instance(self);
        self_
            .list
            .borrow()
            .get(&id)
            .expect("Failed to get expected Supergroup")
            .to_owned()
    }

    pub fn handle_update(&self, update: &Update) {
        if let Update::Supergroup(data) = update {
            let self_ = imp::SupergroupList::from_instance(self);
            let mut list = self_.list.borrow_mut();

            match list.entry(data.supergroup.id) {
                Entry::Occupied(entry) => entry.get().handle_update(update),
                Entry::Vacant(entry) => {
                    let supergroup = Supergroup::from_td_object(&data.supergroup);
                    entry.insert(supergroup);

                    drop(list);

                    let position = (self_.list.borrow().len() - 1) as u32;
                    self.items_changed(position, 0, 1);
                }
            }
        }
    }
}
