use std::sync::OnceLock;

use adw::prelude::*;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::model;
use crate::ui;
use crate::ui::MessageBaseExt;

const MAX_REPLY_CHAR_WIDTH: i32 = 18;

const STICKER_SIZE: i32 = 176;
const EMOJI_SIZE: i32 = 112;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/session/content/message_row/sticker.ui")]
    pub(crate) struct MessageSticker {
        pub(super) message: glib::WeakRef<model::Message>,
        #[template_child]
        pub(super) overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub(super) click: TemplateChild<gtk::GestureClick>,
        #[template_child]
        pub(super) sticker: TemplateChild<ui::Sticker>,
        #[template_child]
        pub(super) indicators: TemplateChild<ui::MessageIndicators>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageSticker {
        const NAME: &'static str = "PaplMessageSticker";
        type Type = super::MessageSticker;
        type ParentType = ui::MessageBase;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            klass.set_css_name("messagesticker");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageSticker {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![glib::ParamSpecObject::builder::<model::Message>("message")
                    .explicit_notify()
                    .build()]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "message" => obj.set_message(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message" => self.message.upgrade().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageSticker {}
    impl ui::MessageBaseImpl for MessageSticker {}

    #[gtk::template_callbacks]
    impl MessageSticker {
        #[template_callback]
        fn on_pressed(&self, _n_press: i32, _x: f64, _y: f64) {
            // TODO: animated emoji needs to play
            // effect when someone clicks on it
            self.sticker.play_animation();
        }
    }
}

glib::wrapper! {
    pub(crate) struct MessageSticker(ObjectSubclass<imp::MessageSticker>)
        @extends gtk::Widget, ui::MessageBase;
}

impl ui::MessageBaseExt for MessageSticker {
    type Message = model::Message;

    fn set_message(&self, message: &Self::Message) {
        let imp = self.imp();

        if imp.message.upgrade().as_ref() == Some(message) {
            return;
        }

        imp.message.set(Some(message));

        imp.indicators.set_message(message.upcast_ref());

        if matches!(
            message.reply_to(),
            Some(model::BoxedMessageReplyTo(
                tdlib::enums::MessageReplyTo::Message(_)
            ))
        ) {
            let reply = ui::MessageReply::new(message);
            reply.set_valign(gtk::Align::Start);
            reply.set_max_char_width(MAX_REPLY_CHAR_WIDTH);

            // FIXME: Do not show message reply when message is being deleted
            // Sticker and the reply should be at the opposite sides of the box
            if message.is_outgoing() {
                reply.insert_before(self, Some(&imp.overlay.get()));
            } else {
                reply.insert_after(self, Some(&imp.overlay.get()));
            }
        }

        let (sticker, looped, is_emoji) = match message.content().0 {
            tdlib::enums::MessageContent::MessageSticker(data) => {
                let sticker = data.sticker;
                (sticker, true, false)
            }
            tdlib::enums::MessageContent::MessageAnimatedEmoji(data) => {
                let sticker = data.animated_emoji.sticker.unwrap();
                let looped = matches!(
                    sticker.full_type,
                    tdlib::enums::StickerFullType::CustomEmoji(_)
                );
                (sticker, looped, true)
            }
            _ => unreachable!(),
        };

        // TODO: that should be handled a bit better in the future
        match &sticker.full_type {
            tdlib::enums::StickerFullType::CustomEmoji(data) if data.needs_repainting => {
                self.add_css_class("needs-repainting")
            }
            _ => self.remove_css_class("needs-repainting"),
        }

        let (size, margin_bottom) = if is_emoji {
            (EMOJI_SIZE, 8)
        } else {
            (STICKER_SIZE, 0)
        };

        imp.sticker.set_longer_side_size(size);
        imp.sticker.set_margin_bottom(margin_bottom);

        imp.sticker
            .update_sticker(sticker, looped, message.chat_().session_());

        self.notify("message");
    }
}
