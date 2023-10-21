use gettextrs::gettext;
use glib::closure;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::Row)]
    #[template(resource = "/app/drey/paper-plane/ui/session/sidebar/chat_folder/row.ui")]
    pub(crate) struct Row {
        #[property(get, set)]
        pub(super) chat_list: glib::WeakRef<model::ChatList>,
        #[template_child]
        pub(super) title_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "PaplSidebarChatFolderRow";
        type Type = super::Row;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("chatfolderrow");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
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

            let obj = &*self.obj();

            let chat_list_expr = Self::Type::this_expression("chat-list");

            gtk::ClosureExpression::new::<String>(
                [
                    &chat_list_expr.chain_property::<model::ChatList>("list-type"),
                    &chat_list_expr.chain_property::<model::ChatList>("title"),
                ],
                closure!(
                    |_: Self::Type, list_type: model::BoxedChatListType, title: String| {
                        use tdlib::enums::ChatList;

                        match list_type.0 {
                            ChatList::Main => gettext("All Chats"),
                            ChatList::Archive => gettext("Archived Chats"),
                            _ => title,
                        }
                    }
                ),
            )
            .bind(&self.title_label.get(), "label", Some(obj));
        }

        fn dispose(&self) {
            utils::unparent_children(&*self.obj());
        }
    }

    impl WidgetImpl for Row {}
}

glib::wrapper! {
    pub(crate) struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}

impl From<&model::ChatList> for Row {
    fn from(chat_list: &model::ChatList) -> Self {
        glib::Object::builder()
            .property("chat-list", chat_list)
            .build()
    }
}
