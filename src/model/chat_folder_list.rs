use std::cell::Cell;
use std::cell::RefCell;
use std::sync::OnceLock;

use gio::subclass::prelude::*;
use glib::Properties;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use crate::model;
use crate::types::ChatFolderId;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::ChatFolderList)]

    pub(crate) struct ChatFolderList {
        pub(super) list: RefCell<Vec<model::ChatList>>,
        #[property(get, set, construct_only)]
        pub(super) session: glib::WeakRef<model::ClientStateSession>,
        #[property(get, set, construct_only)]
        pub(super) main_chat_list: glib::WeakRef<model::ChatList>,
        #[property(get)]
        pub(super) main_chat_list_position: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatFolderList {
        const NAME: &'static str = "ChatFolderList";
        type Type = super::ChatFolderList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ChatFolderList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                Self::derived_properties()
                    .iter()
                    .cloned()
                    .chain(Some(
                        glib::ParamSpecBoolean::builder("has-folders")
                            .read_only()
                            .build(),
                    ))
                    .collect::<Vec<_>>()
            })
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "has-folders" => self.obj().has_folders().to_value(),
                _ => self.derived_property(id, pspec),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj()
                .connect_items_changed(|obj, _, _, _| obj.notify("has-folders"));
        }
    }

    impl ListModelImpl for ChatFolderList {
        fn item_type(&self) -> glib::Type {
            model::ChatList::static_type()
        }

        fn n_items(&self) -> u32 {
            self.list.borrow().len() as u32 + 1
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            let obj = self.obj();

            if obj.main_chat_list_position() == position as i32 {
                obj.main_chat_list().and_upcast()
            } else {
                self.list
                    .borrow()
                    .get(if (position as i32) < obj.main_chat_list_position() {
                        position
                    } else {
                        position - 1
                    } as usize)
                    .map(|item| item.upcast_ref())
                    .cloned()
            }
        }
    }

    impl ChatFolderList {
        pub(super) fn set_main_chat_list_position(&self, position: i32) {
            let obj = self.obj();
            if obj.main_chat_list_position() == position {
                return;
            }

            self.main_chat_list_position.set(position);
            obj.notify_main_chat_list_position();
        }
    }
}

glib::wrapper! {
    pub(crate) struct ChatFolderList(ObjectSubclass<imp::ChatFolderList>) @implements gio::ListModel;
}

impl From<&model::ClientStateSession> for ChatFolderList {
    fn from(session: &model::ClientStateSession) -> Self {
        glib::Object::builder()
            .property("session", session)
            .property("main-chat-list", session.main_chat_list())
            .build()
    }
}

impl ChatFolderList {
    pub(crate) fn has_folders(&self) -> bool {
        self.n_items() > 1
    }

    pub(crate) fn session_(&self) -> model::ClientStateSession {
        self.session().unwrap()
    }

    pub(crate) fn get_or_create(&self, id: ChatFolderId) -> model::ChatList {
        let mut list = self.imp().list.borrow_mut();
        list.iter()
            .find(|chat_list| chat_list.list_type().chat_folder_id().unwrap() == id)
            .cloned()
            .unwrap_or_else(|| {
                let chat_list = model::ChatList::new(
                    &self.session_(),
                    tdlib::enums::ChatList::Folder(tdlib::types::ChatListFolder {
                        chat_folder_id: id,
                    }),
                );

                list.push(chat_list.to_owned());
                let position = list.len() as u32;

                drop(list);

                self.items_changed(position, 0, 1);

                chat_list
            })
    }

    fn internal_get_or_create(&self, id: ChatFolderId, position: u32) -> model::ChatList {
        let imp = self.imp();

        let mut list = imp.list.borrow_mut();

        match list
            .iter()
            .cloned()
            .enumerate()
            .find(|(_, chat_list)| chat_list.list_type().chat_folder_id().unwrap() == id)
        {
            Some((old_position, chat_list)) => {
                if position as usize != old_position {
                    list.remove(old_position);
                    drop(list);
                    self.delegate_items_changed(old_position as u32, 1, 0);

                    let mut order = imp.list.borrow_mut();
                    order.insert(position as usize, chat_list.clone());

                    drop(order);

                    self.delegate_items_changed(position, 0, 1);
                }

                chat_list
            }
            None => {
                let chat_list = model::ChatList::new(
                    &self.session_(),
                    tdlib::enums::ChatList::Folder(tdlib::types::ChatListFolder {
                        chat_folder_id: id,
                    }),
                );

                list.insert(position as usize, chat_list.to_owned());

                drop(list);

                self.delegate_items_changed(position, 0, 1);

                chat_list
            }
        }
    }

    fn delegate_items_changed(&self, position: u32, removed: u32, added: u32) {
        self.items_changed(
            if (position as i32) < self.main_chat_list_position() {
                position
            } else {
                position + 1
            },
            removed,
            added,
        );
    }

    pub(crate) fn handle_update(&self, update: tdlib::enums::Update) {
        if let tdlib::enums::Update::ChatFolders(update) = update {
            let imp = self.imp();

            let mut order = self.imp().list.borrow_mut();

            let mut removed_positions = Vec::new();
            order
                .iter()
                .map(|chat_list| chat_list.list_type().chat_folder_id().unwrap())
                .collect::<Vec<_>>()
                .iter()
                .enumerate()
                .for_each(|(position, old_id)| {
                    if !update
                        .chat_folders
                        .iter()
                        .map(|info| info.id)
                        .any(|new_id| &new_id == old_id)
                    {
                        order.remove(position);
                        removed_positions.push(position);
                    }
                });

            drop(order);

            removed_positions.into_iter().for_each(|position| {
                self.delegate_items_changed(position as u32, 1, 0);
            });

            update
                .chat_folders
                .into_iter()
                .enumerate()
                .for_each(|(position, info)| {
                    let chat_list = self.internal_get_or_create(info.id, position as u32);

                    chat_list.set_title(info.title.as_str());
                    chat_list.set_icon(info.icon.name.as_str());
                });

            imp.set_main_chat_list_position(update.main_chat_list_position);
        }
    }
}
