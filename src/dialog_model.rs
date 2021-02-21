use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::gio;
use gtk::glib;

use crate::dialog_data::DialogData;

mod imp {
    use super::*;
    use glib::subclass;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct DialogModel(pub RefCell<Vec<DialogData>>);

    impl ObjectSubclass for DialogModel {
        const NAME: &'static str = "DialogModel";
        type Type = super::DialogModel;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self(RefCell::new(Vec::new()))
        }
    }

    impl ObjectImpl for DialogModel {}

    impl ListModelImpl for DialogModel {
        fn get_item_type(&self, _list_model: &Self::Type) -> glib::Type {
            DialogData::static_type()
        }

        fn get_n_items(&self, _list_model: &Self::Type) -> u32 {
            self.0.borrow().len() as u32
        }

        fn get_item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.0
                .borrow()
                .get(position as usize)
                .map(|o| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct DialogModel(ObjectSubclass<imp::DialogModel>)
        @implements gio::ListModel;
}

impl DialogModel {
    pub fn new() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create DialogModel")
    }

    pub fn append(&self, obj: &DialogData) {
        let self_ = imp::DialogModel::from_instance(self);
        let index = {
            let mut data = self_.0.borrow_mut();
            data.push(obj.clone());
            data.len() - 1
        };
        self.items_changed(index as u32, 0, 1);
    }

    pub fn remove(&self, index: u32) {
        let self_ = imp::DialogModel::from_instance(self);
        self_.0.borrow_mut().remove(index as usize);
        self.items_changed(index, 1, 0);
    }
}
