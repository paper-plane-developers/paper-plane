use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::tdlib::{Chat, User};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use crate::session::components::Avatar;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="SidebarSearchItemRow" parent="GtkWidget">
        <child>
          <object class="ComponentsAvatar" id="avatar">
            <property name="size">32</property>
            <binding name="item">
              <lookup name="item">SidebarSearchItemRow</lookup>
            </binding>
          </object>
        </child>
        <child>
          <object class="GtkLabel" id="label">
            <property name="ellipsize">end</property>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    pub(crate) struct ItemRow {
        /// A `Chat` or `User`
        pub(super) item: RefCell<Option<glib::Object>>,
        #[template_child]
        pub(super) avatar: TemplateChild<Avatar>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemRow {
        const NAME: &'static str = "SidebarSearchItemRow";
        type Type = super::ItemRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_css_name("sidebarsearchitemrow");
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ItemRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "item",
                    "Item",
                    "The item of this row",
                    glib::Object::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "item" => obj.set_item(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.avatar.unparent();
            self.label.unparent();
        }
    }

    impl WidgetImpl for ItemRow {}
}

glib::wrapper! {
    pub(crate) struct ItemRow(ObjectSubclass<imp::ItemRow>)
        @extends gtk::Widget;
}

impl ItemRow {
    pub(crate) fn new(item: &Option<glib::Object>) -> Self {
        glib::Object::new(&[("item", item)]).expect("Failed to create SidebarSearchItemRow")
    }

    pub(crate) fn set_item(&self, item: Option<glib::Object>) {
        if self.item() == item {
            return;
        }

        let imp = self.imp();

        if let Some(chat) = item.as_ref().and_then(|i| i.downcast_ref::<Chat>()) {
            imp.label.set_label(&chat.title());
        } else if let Some(user) = item.as_ref().and_then(|i| i.downcast_ref::<User>()) {
            imp.label
                .set_label(&(user.first_name() + " " + &user.last_name()));
        } else {
            imp.label.set_label("");

            if let Some(ref item) = item {
                log::warn!("Unexpected item type {:?}", item);
            }
        }

        imp.item.replace(item);
        self.notify("item");
    }

    pub(crate) fn item(&self) -> Option<glib::Object> {
        self.imp().item.borrow().clone()
    }
}
