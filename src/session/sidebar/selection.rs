use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default)]
    pub(crate) struct Selection {
        pub(super) model: RefCell<Option<gio::ListModel>>,
        pub(super) item: RefCell<Option<glib::Object>>,
        pub(super) hide_selection: Cell<bool>,
        pub(super) item_position: Cell<u32>,
        pub(super) signal_handler: RefCell<Option<glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Selection {
        const NAME: &'static str = "SidebarSelection";
        type Type = super::Selection;
        type Interfaces = (gio::ListModel, gtk::SelectionModel);
    }

    impl ObjectImpl for Selection {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "model",
                        "Model",
                        "The model being wrapped",
                        gio::ListModel::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "selected-item",
                        "Selected item",
                        "The selected item",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "hide-selection",
                        "Hide selection",
                        "Whether to hide the selection or not",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "model" => obj.set_model(value.get().unwrap()),
                "selected-item" => obj.set_selected_item(value.get().unwrap()),
                "hide-selection" => obj.set_hide_selection(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "model" => obj.model().to_value(),
                "selected-item" => obj.selected_item().to_value(),
                "hide-selection" => obj.hide_selection().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.item_position.set(gtk::INVALID_LIST_POSITION)
        }

        fn dispose(&self) {
            self.obj().disconnect_model_signal();
        }
    }

    impl ListModelImpl for Selection {
        fn item_type(&self) -> glib::Type {
            glib::Object::static_type()
        }

        fn n_items(&self) -> u32 {
            self.model
                .borrow()
                .as_ref()
                .map(|m| m.n_items())
                .unwrap_or_default()
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.model.borrow().as_ref().and_then(|m| m.item(position))
        }
    }

    impl SelectionModelImpl for Selection {
        fn is_selected(&self, position: u32) -> bool {
            let item_position = self.item_position.get();
            if self.obj().hide_selection() || item_position == gtk::INVALID_LIST_POSITION {
                return false;
            }

            position == item_position
        }

        fn selection_in_range(&self, _position: u32, _n_items: u32) -> gtk::Bitset {
            let result = gtk::Bitset::new_empty();
            let item_position = self.item_position.get();
            if !self.obj().hide_selection() && item_position != gtk::INVALID_LIST_POSITION {
                result.add(item_position);
            }

            result
        }
    }
}

glib::wrapper! {
    // TODO: This is basically https://gitlab.gnome.org/GNOME/libadwaita/-/merge_requests/504,
    // so when that selection model will arrive in libadwaita we should use that instead
    pub(crate) struct Selection(ObjectSubclass<imp::Selection>)
        @implements gio::ListModel, gtk::SelectionModel;
}

impl Selection {
    fn find_item_position(&self, item: &glib::Object, start_pos: u32, end_pos: u32) -> u32 {
        if let Some(model) = self.model() {
            for pos in start_pos..end_pos {
                let check = model.item(pos);

                if check.as_ref() == Some(item) {
                    return pos;
                }
            }
        }

        gtk::INVALID_LIST_POSITION
    }

    fn model_items_changed(&self, position: u32, removed: u32, added: u32) {
        let imp = self.imp();
        let item_position = imp.item_position.get();

        if let Some(selected_item) = self.selected_item() {
            if item_position == gtk::INVALID_LIST_POSITION {
                // Maybe the item got newly added
                imp.item_position.set(self.find_item_position(
                    &selected_item,
                    position,
                    position + added,
                ));
            } else if item_position < position {
                // Nothing to do, position stays the same
            } else if item_position < position + removed {
                imp.item_position.set(self.find_item_position(
                    &selected_item,
                    position,
                    position + added,
                ));
            } else {
                imp.item_position.set(item_position + (added - removed));
            }
        }

        self.items_changed(position, removed, added);
    }

    fn disconnect_model_signal(&self) {
        if let Some(model) = self.model() {
            let handler = self.imp().signal_handler.take().unwrap();
            model.disconnect(handler);
        }
    }

    pub(crate) fn model(&self) -> Option<gio::ListModel> {
        self.imp().model.borrow().clone()
    }

    pub(crate) fn set_model(&self, model: Option<gio::ListModel>) {
        if self.model() == model {
            return;
        }

        let n_items_before = self.n_items();
        self.disconnect_model_signal();

        let imp = self.imp();
        let n_items = if let Some(ref model) = model {
            let handler =
                model.connect_items_changed(clone!(@weak self as obj => move |_, p, r, a| {
                    obj.model_items_changed(p, r, a);
                }));
            imp.signal_handler.replace(Some(handler));

            model.n_items()
        } else {
            0
        };

        imp.model.replace(model);

        self.items_changed(0, n_items_before, n_items);

        self.notify("model");
    }

    pub(crate) fn selected_item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().clone()
    }

    fn set_selected_item_internal(&self, item: Option<glib::Object>, position: u32) {
        let imp = self.imp();

        imp.item.replace(item);

        let old_position = imp.item_position.get();
        imp.item_position.set(position);

        if !self.hide_selection()
            && (old_position != gtk::INVALID_LIST_POSITION
                || position != gtk::INVALID_LIST_POSITION)
        {
            if old_position == gtk::INVALID_LIST_POSITION {
                self.selection_changed(position, 1);
            } else if position == gtk::INVALID_LIST_POSITION {
                self.selection_changed(old_position, 1);
            } else if position < old_position {
                self.selection_changed(position, old_position - position + 1);
            } else {
                self.selection_changed(old_position, position - old_position + 1);
            }
        }

        self.notify("selected-item");
    }

    pub(crate) fn set_selected_item(&self, selected_item: Option<glib::Object>) {
        if self.selected_item() == selected_item {
            return;
        }

        let position = selected_item
            .as_ref()
            .map(|i| self.find_item_position(i, 0, self.n_items()))
            .unwrap_or(gtk::INVALID_LIST_POSITION);

        self.set_selected_item_internal(selected_item, position);
    }

    pub(crate) fn set_selected_position(&self, position: u32) {
        let item = self.item(position);
        self.set_selected_item_internal(item, position);
    }

    pub(crate) fn hide_selection(&self) -> bool {
        self.imp().hide_selection.get()
    }

    pub(crate) fn set_hide_selection(&self, hide_selection: bool) {
        if self.hide_selection() == hide_selection {
            return;
        }

        let imp = self.imp();

        imp.hide_selection.set(hide_selection);
        self.notify("hide-selection");

        let item_position = imp.item_position.get();
        if item_position != gtk::INVALID_LIST_POSITION {
            self.selection_changed(item_position, 1);
        }
    }
}
