use gtk::glib;
use gtk::graphene;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::Background;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(string = r#"
        template $ComponentsThemePreview {
            $ContentBackground background {
                styles ["card"]
                overflow: hidden;
                thumbnail-mode: true;
            }

            Label label {
                styles ["title-1"]
            }
        }
    "#)]
    pub struct ThemePreview {
        #[template_child]
        pub(super) background: TemplateChild<Background>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThemePreview {
        const NAME: &'static str = "ComponentsThemePreview";
        type Type = super::ThemePreview;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("themepreview");
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ThemePreview {
        fn dispose(&self) {
            self.background.unparent();
            self.label.unparent();
        }
    }

    impl WidgetImpl for ThemePreview {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            _ = self.background.measure(orientation, for_size);
            _ = self.label.measure(orientation, for_size);
            if orientation == gtk::Orientation::Vertical {
                (110, 110, -1, -1)
            } else {
                (80, 80, -1, -1)
            }
        }

        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::ConstantSize
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.background.allocate(width, height, baseline, None);

            let label_transform =
                gsk::Transform::new().translate(&graphene::Point::new(0.0, height as f32 - 40.0));

            self.label
                .allocate(width, 40, baseline, Some(label_transform));
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);

            let widget = self.obj();

            let width = widget.width() as f32;
            let height = widget.height() as f32;

            let msg_height = 24.0;
            let msg_width = 46.0;
            let msg_margins = 8.0;

            let gradient_bounds = graphene::Rect::new(0.0, 0.0, width, height);

            let outgoing_message_bounds = graphene::Rect::new(
                width - msg_width - msg_margins,
                msg_margins,
                msg_width,
                msg_height,
            );

            let message_node = self
                .background
                .message_bg_node(&outgoing_message_bounds, &outgoing_message_bounds);

            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(outgoing_message_bounds, 16.0));
            snapshot.append_node(message_node);
            snapshot.pop();

            let ingoing_message_bounds = graphene::Rect::new(
                msg_margins,
                msg_margins * 2.0 + msg_height,
                msg_width,
                msg_height,
            );

            let message_node = self
                .background
                .bg_node(&ingoing_message_bounds, &gradient_bounds);

            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(ingoing_message_bounds, 16.0));

            snapshot.append_color(&widget.color(), &ingoing_message_bounds);
            snapshot.push_opacity(0.1);
            snapshot.append_node(message_node);
            snapshot.pop();
            snapshot.pop();
        }
    }
}

glib::wrapper! {
    pub struct ThemePreview(ObjectSubclass<imp::ThemePreview>)
        @extends gtk::Widget;
}

impl ThemePreview {
    pub(crate) fn new(session: Option<&crate::Session>) -> Self {
        Self::from_chat_theme(crate::utils::default_theme(), session)
    }

    pub(crate) fn from_chat_theme(
        chat_theme: tdlib::types::ChatTheme,
        session: Option<&crate::Session>,
    ) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().label.set_label(&chat_theme.name);
        obj.imp().background.set_chat_theme(Some(chat_theme));
        if let Some(session) = session {
            obj.imp().background.set_session(session);
        }
        obj
    }
}
