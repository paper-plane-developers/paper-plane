use std::cell::Cell;
use std::cell::RefCell;

use gtk::glib;
use gtk::graphene;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

const SNOW_SHADER: &[u8] = r#"
uniform float u_time;
uniform vec4 u_color;

float random(vec2 co) {
   return fract(sin(dot(co.xy,vec2(12.9898,78.233))) * 43758.5453);
}

float map(vec2 p) {
    float w = 0.05;
    p.y -= u_time * 0.05;
    vec2 idx = floor(p / w * 0.5 + 0.5);
    p.x += cos(u_time * random(idx)) * 0.01;
    float r = sin(random(idx)) * 0.005 + 0.001;
    p = mod(p + w, w * 2.) - w;
    float d = length(p - 0.03 * vec2(cos(idx.y), cos(idx.x))) - r;
    float color = d < 0.0 ? 0.5 : 0.0;
    color *= (sin(u_time * random(idx)) + 1.0) / 2.0;
	return color;
}

void mainImage(out vec4 fragColor,
               in vec2 fragCoord,
               in vec2 resolution,
               in vec2 uv) {

    float sum = 0;
    // anti-aliasing
    for (float i = -0.5; i < 1.5; i += 0.2) {
        for (float j = -0.5; j < 1.5; j += 0.2) {
            vec2 uv2 = (fragCoord.xy + vec2(i, j)) / resolution.yy * 0.2;
            sum += map(uv2);
        }
    }
    sum /= 25;

	fragColor = u_color * sum;
}
"#
.as_bytes();

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Snow {
        pub(super) snow_shader: RefCell<Option<gsk::GLShader>>,
        pub(super) time_start: Cell<i64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Snow {
        const NAME: &'static str = "PaplSnow";
        type Type = super::Snow;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for Snow {
        fn constructed(&self) {
            if super::Snow::correct_date() {
                self.obj().add_tick_callback(|widget, _clock| {
                    widget.queue_draw();
                    glib::ControlFlow::Continue
                });
            } else {
                self.obj().set_visible(false);
            }
        }
    }

    impl WidgetImpl for Snow {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            widget.ensure_shader();

            if let Some(shader) = &*self.snow_shader.borrow() {
                let frame_time = widget.frame_clock().unwrap().frame_time();
                let time = (frame_time - self.time_start.get()) as f32 / 1000000.0;

                let color = self.obj().color();
                let color =
                    graphene::Vec4::new(color.red(), color.green(), color.blue(), color.alpha());
                let args_builder = gsk::ShaderArgsBuilder::new(shader, None);

                args_builder.set_float(0, time);
                args_builder.set_vec4(1, &color);

                let wigth = widget.width() as f32;
                let height = widget.height() as f32;
                let bounds = graphene::Rect::new(0.0, 0.0, wigth, height);

                snapshot.push_gl_shader(shader, &bounds, args_builder.to_args());
                snapshot.pop();
            }
        }
    }
}

glib::wrapper! {
    pub struct Snow(ObjectSubclass<imp::Snow>)
        @extends gtk::Widget;
}

impl Snow {
    fn correct_date() -> bool {
        match glib::DateTime::now_local() {
            Ok(date_time) => {
                let month = date_time.month();
                let day = date_time.day_of_month();
                match month {
                    12 => day >= 24,
                    1 => day <= 5,
                    _ => false,
                }
            }
            Err(_) => false,
        }
    }

    fn ensure_shader(&self) {
        let imp = self.imp();
        if imp.snow_shader.borrow().is_none() {
            let renderer = self.native().unwrap().renderer().unwrap();

            let shader = {
                let bytes = glib::Bytes::from_static(SNOW_SHADER);
                let shader = gsk::GLShader::from_bytes(&bytes);
                match shader.compile(&renderer) {
                    Err(e) => {
                        // That shader isn't important
                        log::error!("Couldn't compile snow shader: {}", e);
                        self.set_visible(false);
                        None
                    }
                    Ok(_) => Some(shader),
                }
            };

            imp.snow_shader.replace(shader);
            imp.time_start.set(self.frame_clock().unwrap().frame_time());
        }
    }
}
