use adw::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use tdlib::enums::{MessageContent, StickerFullType};

use crate::components::Sticker;
use crate::session::content::message_row::{
    MessageBase, MessageBaseImpl, MessageIndicators, MessageReply,
};
use crate::tdlib::Message;

use super::base::MessageBaseExt;

const MAX_REPLY_CHAR_WIDTH: i32 = 18;

const STICKER_SIZE: i32 = 176;
const EMOJI_SIZE: i32 = 112;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = r#"
    template MessageSticker : .MessageBase {
        layout-manager: BoxLayout {};

        Overlay overlay {
            GestureClick click {
                button: 1;

                released => on_pressed() swapped;
            }

            .ComponentsSticker sticker {}

            [overlay]
            .MessageIndicators indicators {
            halign: end;
            valign: end;
            }
        }
    }
    "#)]
    pub(crate) struct MessageSticker {
        pub(super) message: RefCell<Option<Message>>,
        #[template_child]
        pub(super) overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub(super) click: TemplateChild<gtk::GestureClick>,
        #[template_child]
        pub(super) sticker: TemplateChild<Sticker>,
        #[template_child]
        pub(super) indicators: TemplateChild<MessageIndicators>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageSticker {
        const NAME: &'static str = "MessageSticker";
        type Type = super::MessageSticker;
        type ParentType = MessageBase;

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
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<Message>("message")
                    .explicit_notify()
                    .build()]
            });
            PROPERTIES.as_ref()
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
                "message" => self.message.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for MessageSticker {}
    impl MessageBaseImpl for MessageSticker {}

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
        @extends gtk::Widget, MessageBase;
}

impl MessageBaseExt for MessageSticker {
    type Message = Message;

    fn set_message(&self, message: Self::Message) {
        let imp = self.imp();

        if imp.message.borrow().as_ref() == Some(&message) {
            return;
        }

        imp.message.replace(Some(message));

        let message_ref = imp.message.borrow();
        let message = message_ref.as_ref().unwrap();

        imp.indicators.set_message(message.clone().upcast());

        if message.reply_to_message_id() != 0 {
            let reply = MessageReply::new(message);
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
            MessageContent::MessageSticker(data) => {
                let sticker = data.sticker;
                (sticker, true, false)
            }
            MessageContent::MessageAnimatedEmoji(data) => {
                let sticker = data.animated_emoji.sticker.unwrap();
                let looped = matches!(sticker.full_type, StickerFullType::CustomEmoji(_));
                (sticker, looped, true)
            }
            _ => unreachable!(),
        };

        let (size, margin_bottom) = if is_emoji {
            (EMOJI_SIZE, 8)
        } else {
            (STICKER_SIZE, 0)
        };

        imp.sticker.set_longer_side_size(size);
        imp.sticker.set_margin_bottom(margin_bottom);

        imp.sticker
            .update_sticker(sticker, looped, message.chat().session());

        self.notify("message");
    }
}
