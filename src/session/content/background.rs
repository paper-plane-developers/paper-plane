use std::cell::Cell;
use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::clone;
use glib::once_cell::unsync::OnceCell;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::graphene;
use gtk::gsk;

const GRADIENT_SHADER: &[u8] = r#"
// That shader was taken from Telegram for android source
// https://github.com/DrKLO/Telegram/commit/2112affb2e4941334f8fbc3944385806b3c4e3d6#diff-dfdd1e8c4691747fd30199b7a2f5041a126b23e1450b29afe441eb0ebed01c68

precision highp float;

uniform vec3 color1;
uniform vec3 color2;
uniform vec3 color3;
uniform vec3 color4;
uniform vec4 p12;
uniform vec4 p34;
uniform vec4 gradient_bounds;

void mainImage(out vec4 fragColor,
               in vec2 fragCoord,
               in vec2 resolution,
               in vec2 uv) {
    vec2 p1 = p12.xy;
    vec2 p2 = p12.zw;
    vec2 p3 = p34.xy;
    vec2 p4 = p34.zw;

    uv = (fragCoord - gradient_bounds.xy) / gradient_bounds.zw;
    uv.y = 1.0 - uv.y;

    float dp1 = distance(uv, p1);
    float dp2 = distance(uv, p2);
    float dp3 = distance(uv, p3);
    float dp4 = distance(uv, p4);
    float minD = min(dp1, min(dp2, min(dp3, dp4)));
    float p = 5.0;
    dp1 = pow(1.0 - (dp1 - minD), p);
    dp2 = pow(1.0 - (dp2 - minD), p);
    dp3 = pow(1.0 - (dp3 - minD), p);
    dp4 = pow(1.0 - (dp4 - minD), p);
    float sumDp = dp1 + dp2 + dp3 + dp4;

    vec3 color = (color1 * dp1 + color2 * dp2 + color3 * dp3 + color4 * dp4) / sumDp;
    fragColor = vec4(color, 1.0);
}
"#
.as_bytes();

mod imp {
    use super::*;

    #[derive(Default)]
    pub(crate) struct Background {
        pub(super) background_texture: RefCell<Option<gdk::Texture>>,

        pub(super) last_size: Cell<(f32, f32)>,

        pub(super) shader: RefCell<Option<gsk::GLShader>>,
        pub(super) pattern: OnceCell<gdk::Texture>,

        pub(super) animation: OnceCell<adw::Animation>,
        pub(super) progress: Cell<f32>,
        pub(super) phase: Cell<u32>,

        pub(super) dark: Cell<bool>,

        pub(super) bg_colors: RefCell<Vec<graphene::Vec3>>,
        pub(super) message_colors: RefCell<Vec<graphene::Vec3>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Background {
        const NAME: &'static str = "ContentBackground";
        type Type = super::Background;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for Background {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let pattern = gdk::Texture::from_resource("/app/drey/paper-plane/images/pattern.svg");

            self.pattern.set(pattern).unwrap();

            let style_manager = adw::StyleManager::default();
            obj.set_theme(hard_coded_themes(style_manager.is_dark()));

            style_manager.connect_dark_notify(clone!(@weak obj => move |style_manager| {
                obj.set_theme(hard_coded_themes(style_manager.is_dark()))
            }));

            if style_manager.is_high_contrast() {
                obj.add_css_class("fallback");
            }

            style_manager.connect_high_contrast_notify(clone!(@weak obj => move |style_manager| {
                if style_manager.is_high_contrast() {
                    obj.add_css_class("fallback");
                } else if obj.imp().shader.borrow().is_some() {
                    obj.remove_css_class("fallback");
                }
            }));

            let target = adw::CallbackAnimationTarget::new(clone!(@weak obj => move |progress| {
                let imp = obj.imp();
                imp.background_texture.take();
                let progress = progress as f32;
                if progress >= 1.0 {
                    imp.progress.set(0.0);
                    imp.phase.set((imp.phase.get() + 1) % 8);
                } else {
                    imp.progress.set(progress)
                }
                obj.queue_draw();
            }));

            let animation = adw::TimedAnimation::builder()
                .widget(&*obj)
                .value_from(0.0)
                .value_to(1.0)
                .duration(200)
                .target(&target)
                .easing(adw::Easing::EaseInOutQuad)
                .build()
                .upcast();

            self.animation.set(animation).unwrap();
        }
    }

    impl WidgetImpl for Background {
        fn realize(&self) {
            self.parent_realize();
            self.obj().ensure_shader();
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();

            if widget.has_css_class("fallback") {
                // fallback code
                if let Some(child) = widget.child() {
                    widget.snapshot_child(&child, snapshot);
                }
                return;
            };

            let width = widget.width() as f32;
            let height = widget.height() as f32;

            if width == 0.0 || height == 0.0 {
                return;
            }

            let bounds = graphene::Rect::new(0.0, 0.0, width, height);

            let size_changed = self.last_size.replace((width, height)) != (width, height);

            self.snapshot_gradient(snapshot, &bounds, size_changed);

            self.snapshot_pattern(snapshot, &bounds);
        }
    }

    impl BinImpl for Background {}

    impl Background {
        fn snapshot_gradient(
            &self,
            snapshot: &gtk::Snapshot,
            bounds: &graphene::Rect,
            size_changed: bool,
        ) {
            if self.progress.get() == 0.0 {
                let texture = match self.background_texture.take() {
                    Some(texture) if !size_changed => texture,
                    _ => {
                        let renderer = self.obj().native().unwrap().renderer();

                        renderer.render_texture(self.obj().bg_node(bounds, bounds), Some(bounds))
                    }
                };

                snapshot.append_texture(&texture, bounds);
                self.background_texture.replace(Some(texture));
            } else {
                snapshot.append_node(&self.obj().bg_node(bounds, bounds));
            }
        }

        fn snapshot_pattern(&self, snapshot: &gtk::Snapshot, bounds: &graphene::Rect) {
            let widget = self.obj();
            let pattern = self.pattern.get().unwrap();

            let pattern_bounds = graphene::Rect::new(
                0.0,
                0.0,
                pattern.width() as f32 * 0.3,
                pattern.height() as f32 * 0.3,
            );

            let mut matrix = [0.0; 16];
            let mut offset = [0.0; 4];
            if self.dark.get() {
                matrix[15] = -0.3;
                offset = [0.08; 4];
                offset[3] = 1.0;
            } else {
                matrix[15] = 0.1;
            }
            let color_matrix = graphene::Matrix::from_float(matrix);
            let color_offset = graphene::Vec4::from_float(offset);

            snapshot.push_color_matrix(&color_matrix, &color_offset);
            snapshot.push_repeat(bounds, Some(&pattern_bounds));
            snapshot.append_texture(pattern, &pattern_bounds);
            snapshot.pop();
            snapshot.pop();

            if let Some(child) = widget.child() {
                widget.snapshot_child(&child, snapshot);
            }
        }

        pub(super) fn gradient_shader_node(
            &self,
            bounds: &graphene::Rect,
            gradient_bounds: &graphene::Rect,
            colors: &[graphene::Vec3],
        ) -> gsk::GLShaderNode {
            let Some(gradient_shader) = &*self.shader.borrow() else {
                unreachable!()
            };

            let args_builder = gsk::ShaderArgsBuilder::new(gradient_shader, None);

            let progress = self.progress.get();
            let phase = self.phase.get() as usize;

            let &[c1, c2, c3, c4] = colors else {
                  unimplemented!("Unexpected color count")
            };

            args_builder.set_vec3(0, &c1);
            args_builder.set_vec3(1, &c2);
            args_builder.set_vec3(2, &c3);
            args_builder.set_vec3(3, &c4);

            let [p12, p34] = Self::calculate_positions(progress, phase);
            args_builder.set_vec4(4, &p12);
            args_builder.set_vec4(5, &p34);

            let gradient_bounds = {
                graphene::Vec4::new(
                    gradient_bounds.x(),
                    gradient_bounds.y(),
                    gradient_bounds.width(),
                    gradient_bounds.height(),
                )
            };

            args_builder.set_vec4(6, &gradient_bounds);

            gsk::GLShaderNode::new(gradient_shader, bounds, &args_builder.to_args(), &[])
        }

        fn calculate_positions(progress: f32, phase: usize) -> [graphene::Vec4; 2] {
            static POSITIONS: [(f32, f32); 8] = [
                (0.80, 0.10),
                (0.60, 0.20),
                (0.35, 0.25),
                (0.25, 0.60),
                (0.20, 0.90),
                (0.40, 0.80),
                (0.65, 0.75),
                (0.75, 0.40),
            ];

            let mut points = [(0.0, 0.0); 4];

            for i in 0..4 {
                let start = POSITIONS[(i * 2 + phase) % 8];
                let end = POSITIONS[(i * 2 + phase + 1) % 8];

                fn interpolate(start: f32, end: f32, value: f32) -> f32 {
                    start + ((end - start) * value)
                }

                let x = interpolate(start.0, end.0, progress);
                let y = interpolate(start.1, end.1, progress);

                points[i] = (x, y);
            }

            let points: Vec<_> = points
                .chunks(2)
                .map(|p| {
                    let [(x1, y1), (x2, y2)]: [(f32, f32); 2] = p.try_into().unwrap();
                    graphene::Vec4::from_float([x1, y1, x2, y2])
                })
                .collect();

            points.try_into().unwrap()
        }
    }
}

glib::wrapper! {
    pub(crate) struct Background(ObjectSubclass<imp::Background>)
        @extends gtk::Widget, adw::Bin;
}

impl Background {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn set_theme(&self, theme: tdlib::types::ThemeSettings) {
        let Some(background) = theme.background else { return; };
        let imp = self.imp();

        imp.dark.set(background.is_dark);

        let bg_fill = match background.r#type {
            tdlib::enums::BackgroundType::Pattern(pattern) => pattern.fill,
            tdlib::enums::BackgroundType::Fill(fill) => fill.fill,
            tdlib::enums::BackgroundType::Wallpaper(_) => {
                unimplemented!("Wallpaper chat background")
            }
        };

        imp.bg_colors.replace(fill_colors(bg_fill));
        imp.message_colors
            .replace(fill_colors(theme.outgoing_message_fill));

        imp.background_texture.take();
        self.queue_draw();
    }

    pub(crate) fn animate(&self) {
        let animation = self.imp().animation.get().unwrap();

        let val = animation.value();
        if val == 0.0 || val == 1.0 {
            animation.play()
        }
    }

    pub(crate) fn subscribe_to_redraw(&self, child: &gtk::Widget) {
        let animation = self.imp().animation.get().unwrap();
        animation.connect_value_notify(clone!(@weak child => move |_| child.queue_draw()));
    }

    pub(crate) fn bg_node(
        &self,
        bounds: &graphene::Rect,
        gradient_bounds: &graphene::Rect,
    ) -> gsk::GLShaderNode {
        self.imp()
            .gradient_shader_node(bounds, gradient_bounds, &self.imp().bg_colors.borrow())
    }

    pub(crate) fn message_bg_node(
        &self,
        bounds: &graphene::Rect,
        gradient_bounds: &graphene::Rect,
    ) -> gsk::GLShaderNode {
        self.imp().gradient_shader_node(
            bounds,
            gradient_bounds,
            &self.imp().message_colors.borrow(),
        )
    }

    fn ensure_shader(&self) {
        let imp = self.imp();
        if imp.shader.borrow().is_none() {
            let renderer = self.native().unwrap().renderer();

            let shader = gsk::GLShader::from_bytes(&GRADIENT_SHADER.into());
            match shader.compile(&renderer) {
                Err(e) => {
                    if !e.matches(gio::IOErrorEnum::NotSupported) {
                        log::error!("can't compile shader for gradient background {e}");
                    }
                    self.add_css_class("fallback");
                }
                Ok(_) => {
                    imp.shader.replace(Some(shader));
                }
            }
        };
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::new()
    }
}

fn fill_colors(fill: tdlib::enums::BackgroundFill) -> Vec<graphene::Vec3> {
    match fill {
        tdlib::enums::BackgroundFill::FreeformGradient(gradient) if gradient.colors.len() == 4 => {
            gradient
                .colors
                .into_iter()
                .map(|int_color| {
                    let [_, r, g, b] = int_color.to_be_bytes();
                    graphene::Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
                })
                .collect()
        }
        _ => unimplemented!("Unsupported background fill: {fill:?}"),
    }
}

fn hard_coded_themes(dark: bool) -> tdlib::types::ThemeSettings {
    fn theme(
        dark: bool,
        bg_colors: Vec<i32>,
        message_colors: Vec<i32>,
    ) -> tdlib::types::ThemeSettings {
        use tdlib::enums::BackgroundFill::*;
        use tdlib::enums::BackgroundType::Fill;
        use tdlib::types::*;

        ThemeSettings {
            background: Some(Background {
                is_default: true,
                is_dark: dark,
                r#type: Fill(BackgroundTypeFill {
                    fill: FreeformGradient(BackgroundFillFreeformGradient { colors: bg_colors }),
                }),
                id: 0,
                name: String::new(),
                document: None,
            }),
            accent_color: 0,
            animate_outgoing_message_fill: false,
            outgoing_message_accent_color: 0,
            outgoing_message_fill: FreeformGradient(BackgroundFillFreeformGradient {
                colors: message_colors,
            }),
        }
    }

    // tr tl bl br

    if dark {
        theme(
            dark,
            vec![0xd6932e, 0xbc40db, 0x4280d7, 0x614ed5],
            vec![0x2d52ab, 0x4036a1, 0x9f388d, 0x9d3941],
        )
    } else {
        theme(
            dark,
            vec![0x94dae9, 0x9aeddb, 0x94c3f6, 0xac96f7],
            vec![0xddecff, 0xe0ddfd, 0xdbffff, 0xddffdf],
        )
    }
}
