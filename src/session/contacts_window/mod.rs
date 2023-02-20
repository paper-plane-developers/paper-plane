mod row;

use row::ContactRow;

use adw::subclass::prelude::*;
use glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use crate::tdlib::User;
use crate::utils::spawn;
use crate::Session;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;

    use crate::strings;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="ContactsWindow" parent="AdwWindow">
        <property name="title" translatable="true">Contacts</property>
        <property name="modal">true</property>
        <property name="default-width">360</property>
        <property name="default-height">600</property>
        <property name="content">
          <object class="AdwToolbarView">
            <child type="top">
              <object class="GtkHeaderBar"/>
            </child>
            <property name="content">
              <object class="GtkScrolledWindow">
                <property name="vexpand">true</property>
                <property name="hscrollbar-policy">never</property>
                <property name="child">
                  <object class="AdwClampScrollable">
                    <property name="child">
                      <object class="GtkListView" id="list_view">
                        <property name="single-click-activate">True</property>
                        <signal name="activate" handler="list_activate" swapped="true"/>
                        <style>
                          <class name="navigation-sidebar"/>
                        </style>
                        <property name="model">
                          <object class="GtkNoSelection">
                            <property name="model">
                              <object class="GtkSortListModel" id="sort_model">
                                <property name="sorter">
                                  <object class="GtkStringSorter">
                                    <property name="expression">
                                      <closure type="gchararray" function="user_display_name"/>
                                    </property>
                                  </object>
                                </property>
                              </object>
                            </property>
                          </object>
                        </property>
                        <property name="factory">
                          <object class="GtkBuilderListItemFactory">
                            <property name="bytes"><![CDATA[
                              <interface>
                                <template class="GtkListItem">
                                  <property name="child">
                                    <object class="ContactRow">
                                      <binding name="user">
                                        <lookup name="item">GtkListItem</lookup>
                                      </binding>
                                    </object>
                                  </property>
                                </template>
                              </interface>
                            ]]></property>
                          </object>
                        </property>
                      </object>
                    </property>
                  </object>
                </property>
              </object>
            </property>
          </object>
        </property>
      </template>
    </interface>
    "#)]
    pub(crate) struct ContactsWindow {
        pub(super) session: OnceCell<Session>,
        #[template_child]
        pub(super) sort_model: TemplateChild<gtk::SortListModel>,
        #[template_child]
        pub(super) list_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContactsWindow {
        const NAME: &'static str = "ContactsWindow";
        type Type = super::ContactsWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            ContactRow::static_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContactsWindow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("contact-activated")
                    .param_types([i64::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for ContactsWindow {}
    impl WindowImpl for ContactsWindow {}
    impl AdwWindowImpl for ContactsWindow {}

    #[gtk::template_callbacks]
    impl ContactsWindow {
        #[template_callback]
        fn list_activate(&self, pos: u32) {
            let obj = self.obj();
            let user = self
                .list_view
                .model()
                .and_then(|model| model.item(pos))
                .and_downcast::<User>()
                .unwrap();

            obj.emit_by_name::<()>("contact-activated", &[&user.id()]);
            obj.close();
        }

        #[template_callback]
        fn user_display_name(user: &User) -> String {
            strings::user_display_name(user, true)
        }
    }
}

glib::wrapper! {
    pub(crate) struct ContactsWindow(ObjectSubclass<imp::ContactsWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl ContactsWindow {
    pub(crate) fn new(parent: Option<&gtk::Window>, session: Session) -> Self {
        let obj: Self = glib::Object::builder()
            .property("transient-for", parent)
            .build();

        obj.imp().session.set(session).unwrap();

        spawn(clone!(@weak obj => async move {
            obj.fetch_contacts().await;
        }));

        obj
    }

    async fn fetch_contacts(&self) {
        let session = self.imp().session.get().unwrap();

        match session.fetch_contacts().await {
            Ok(users) => {
                let list = gio::ListStore::new(User::static_type());
                list.splice(0, 0, &users);

                self.imp().sort_model.set_model(Some(&list));
            }
            Err(e) => {
                log::warn!("Error fetching contacts: {:?}", e)
            }
        }
    }

    pub(crate) fn connect_contact_activated<F: Fn(&Self, i64) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("contact-activated", true, move |values| {
            let obj = values[0].get().unwrap();
            let user_id = values[1].get().unwrap();
            f(obj, user_id);
            None
        })
    }
}
