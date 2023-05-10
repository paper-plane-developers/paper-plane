mod event_row;
mod item;
mod message_row;
mod model;
mod row;

use self::event_row::MessageListViewEventRow;
use self::item::{MessageListViewItem, MessageListViewItemType};
use self::message_row::MessageRow;
use self::model::{MessageListViewModel, MessageListViewModelError};
use self::row::MessageListViewRow;

use adw::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};

use crate::tdlib::{Chat, ChatType, SponsoredMessage};
use crate::utils::spawn;
use crate::Session;

const MIN_N_ITEMS: u32 = 20;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) enum MessageListViewType {
    #[default]
    ChatHistory,
    PinnedMessages,
}

mod imp {
    use super::*;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/message-list-view.ui")]
    pub(crate) struct MessageListView {
        pub(super) is_auto_scrolling: Cell<bool>,
        pub(super) model: RefCell<Option<MessageListViewModel>>,
        pub(super) message_menu: OnceCell<gtk::PopoverMenu>,
        #[template_child]
        pub(super) scroll_to_bottom_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub(super) list_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageListView {
        const NAME: &'static str = "MessageListView";
        type Type = super::MessageListView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            MessageListViewRow::static_type();
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("messagelistview");

            klass.install_action(
                "message-list-view.show-message-menu",
                Some("dd"),
                |widget, _, variant| {
                    let (x, y) = variant.and_then(|v| v.get()).unwrap();
                    widget.show_message_menu(x, y);
                },
            );
            klass.install_action(
                "message-list-view.scroll-to-bottom",
                None,
                |widget, _, _| {
                    widget.scroll_to_bottom();
                },
            );
            klass.install_action_async(
                "message-list-view.revoke-delete",
                None,
                |widget, _, _| async move {
                    widget.show_delete_dialog(true).await;
                },
            );
            klass.install_action_async(
                "message-list-view.delete",
                None,
                |widget, _, _| async move {
                    widget.show_delete_dialog(false).await;
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageListView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let adj = self.list_view.vadjustment().unwrap();
            adj.connect_value_changed(clone!(@weak obj => move |adj| {
                let imp = obj.imp();

                imp.is_auto_scrolling.set(adj.value() + adj.page_size() >= adj.upper());
                imp.scroll_to_bottom_revealer.set_reveal_child(!imp.is_auto_scrolling.get());

                if adj.value() < adj.page_size() * 2.0 || adj.upper() <= adj.page_size() * 2.0 {
                    spawn(clone!(@weak obj => async move {
                        obj.load_older_messages().await;
                    }));
                }
            }));

            adj.connect_upper_notify(clone!(@weak obj => move |_| {
                if obj.imp().is_auto_scrolling.get() {
                    obj.scroll_to_bottom();
                }
            }));
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for MessageListView {
        // fn direction_changed(&self, previous_direction: gtk::TextDirection) {
        //     let obj = self.obj();

        //     if obj.direction() == previous_direction {
        //         return;
        //     }

        //     if let Some(menu) = self.message_menu.get() {
        //         menu.set_halign(if obj.direction() == gtk::TextDirection::Rtl {
        //             gtk::Align::End
        //         } else {
        //             gtk::Align::Start
        //         });
        //     }
        // }
    }
}

glib::wrapper! {
    pub(crate) struct MessageListView(ObjectSubclass<imp::MessageListView>)
        @extends gtk::Widget;
}

impl MessageListView {
    pub(crate) fn load_messages(&self, type_: MessageListViewType, chat: &Chat) {
        let imp = self.imp();
        let model = MessageListViewModel::new(type_, chat);

        // Request sponsored message, if needed
        let list_view_model: gio::ListModel = if matches!(chat.type_(), ChatType::Supergroup(supergroup) if supergroup.is_channel())
        {
            let list = gio::ListStore::new(gio::ListModel::static_type());

            // We need to create a list here so that we can append the sponsored message
            // to the chat history in the GtkListView using a GtkFlattenListModel
            let sponsored_message_list = gio::ListStore::new(SponsoredMessage::static_type());
            list.append(&sponsored_message_list);

            let chat_id = chat.id();
            let session = chat.session();
            spawn(clone!(@weak self as obj => async move {
                obj.request_sponsored_message(&session, chat_id, &sponsored_message_list).await;
            }));

            list.append(&model);

            gtk::FlattenListModel::new(Some(list)).upcast()
        } else {
            model.clone().upcast()
        };

        let selection = gtk::NoSelection::new(Some(list_view_model));
        imp.list_view.set_model(Some(&selection));

        spawn(clone!(@weak self as obj, @weak model => async move {
            obj.load_initial_messages(&model).await;
        }));

        imp.model.replace(Some(model));
    }

    async fn load_initial_messages(&self, model: &MessageListViewModel) {
        while model.n_items() < MIN_N_ITEMS {
            let limit = MIN_N_ITEMS - model.n_items();

            match model.load_older_messages(limit as i32).await {
                Ok(can_load_more) => {
                    if !can_load_more {
                        break;
                    }
                }
                Err(e) => {
                    log::warn!("Couldn't load initial history messages: {}", e);
                    break;
                }
            }
        }
    }

    async fn request_sponsored_message(
        &self,
        session: &Session,
        chat_id: i64,
        list: &gio::ListStore,
    ) {
        match SponsoredMessage::request(chat_id, &session).await {
            Ok(sponsored_message) => {
                if let Some(sponsored_message) = sponsored_message {
                    list.append(&sponsored_message);
                }
            }
            Err(e) => {
                if e.code != 404 {
                    log::warn!("Failed to request a sponsored message: {:?}", e);
                }
            }
        }
    }

    async fn load_older_messages(&self) {
        if let Some(model) = self.imp().model.borrow().as_ref() {
            if let Err(MessageListViewModelError::Tdlib(e)) = model.load_older_messages(20).await {
                log::warn!("Couldn't load more chat messages: {e:?}");
            }
        }
    }

    fn show_message_menu(&self, x: f64, y: f64) {
        let menu = self.imp().message_menu.get_or_init(|| {
            let menu =
                gtk::Builder::from_resource("/com/github/melix99/telegrand/ui/message-menu.ui")
                    .object::<gtk::PopoverMenu>("menu")
                    .unwrap();

            menu.set_halign(if self.direction() == gtk::TextDirection::Rtl {
                gtk::Align::End
            } else {
                gtk::Align::Start
            });
            menu.set_parent(self);

            menu
        });

        menu.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        menu.popup();
    }

    fn scroll_to_bottom(&self) {
        let imp = self.imp();

        imp.is_auto_scrolling.set(true);
        imp.scrolled_window
            .emit_by_name::<bool>("scroll-child", &[&gtk::ScrollType::End, &false]);
    }

    async fn show_delete_dialog(&self, revoke: bool) {
        let parent = self.root().and_downcast::<gtk::Window>().unwrap();
        let body = if revoke {
            gettext("Do you want to delete this message for <b>everyone</b>?")
        } else {
            gettext("Do you want to delete this message?")
        };
        let dialog = adw::MessageDialog::new(
            Some(&parent),
            Some(&gettext("Confirm Message Deletion")),
            Some(&body),
        );

        dialog.set_body_use_markup(true);
        dialog.add_responses(&[("no", &gettext("_No")), ("yes", &gettext("_Yes"))]);
        dialog.set_default_response(Some("no"));
        dialog.set_response_appearance("yes", adw::ResponseAppearance::Destructive);

        dialog.choose_future().await;

        // dialog.choose(
        //     gio::Cancellable::NONE,
        //     clone!(@weak self as obj => move |response| {
        //         if response == "yes" {
        // if let Ok(message) = obj.message().downcast::<Message>() {
        //     spawn(async move {
        //         if let Err(e) = message.delete(revoke).await {
        //             log::warn!("Error deleting a message (revoke = {}): {:?}", revoke, e);
        //         }
        //     });
        // }
        //         }
        //     }));
    }
}
