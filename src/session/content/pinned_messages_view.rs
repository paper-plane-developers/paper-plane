use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::components::{MessageListView, MessageListViewType};
use crate::tdlib::Chat;

mod imp {
    use super::*;
    use glib::Properties;
    use once_cell::unsync::OnceCell;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::PinnedMessagesView)]
    #[template(string = r#"
    using Adw 1;

    template PinnedMessagesView {
        Adw.ToolbarView toolbar_view {
            [top]
            HeaderBar {
                [start]
                Button {
                    icon-name: "go-previous-symbolic";
                    action-name: "content.go-back";
                }
            }

            content: .MessageListView message_list_view {};
        }
    }
    "#)]
    pub(crate) struct PinnedMessagesView {
        #[property(get, set, construct_only)]
        pub(super) chat: OnceCell<Chat>,
        #[template_child]
        pub(super) toolbar_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) message_list_view: TemplateChild<MessageListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PinnedMessagesView {
        const NAME: &'static str = "PinnedMessagesView";
        type Type = super::PinnedMessagesView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PinnedMessagesView {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let chat = self.chat.get().unwrap();
            self.message_list_view
                .load_messages(MessageListViewType::PinnedMessages, chat);
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for PinnedMessagesView {}
}

glib::wrapper! {
    pub(crate) struct PinnedMessagesView(ObjectSubclass<imp::PinnedMessagesView>)
        @extends gtk::Widget;
}

impl PinnedMessagesView {
    pub(crate) fn new(chat: &Chat) -> Self {
        glib::Object::builder().property("chat", chat).build()
    }
}
