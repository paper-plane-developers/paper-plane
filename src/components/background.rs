use std::cell::Cell;
use std::cell::RefCell;
use std::fmt::Write;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::clone;
use glib::once_cell::unsync::OnceCell;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::graphene;
use gtk::gsk;
use once_cell::sync::Lazy;

use crate::utils::spawn;

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

static DEFAULT_PATTERN: Lazy<gdk::Texture> =
    Lazy::new(|| gdk::Texture::from_resource("/app/drey/paper-plane/images/pattern.svg"));

static mut SHADER: Option<gsk::GLShader> = None;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Background)]
    pub(crate) struct Background {
        #[property(get, set)]
        pub(super) thumbnail_mode: Cell<bool>,

        pub(super) settings: OnceCell<gio::Settings>,
        pub(super) session: RefCell<glib::WeakRef<crate::Session>>,
        pub(super) settings_handler: RefCell<Option<glib::SignalHandlerId>>,

        pub(super) chat_theme: RefCell<Option<tdlib::types::ChatTheme>>,

        pub(super) background_cache: RefCell<Option<gdk::Texture>>,

        pub(super) document: RefCell<Option<tdlib::types::Document>>,
        pub(super) document_target_file_id: Cell<i32>,
        pub(super) document_texture: RefCell<Option<gdk::Texture>>,

        pub(super) loaded_documents: RefCell<Vec<(i32, gdk::Texture)>>,

        pub(super) last_size: Cell<(f32, f32)>,

        pub(super) pattern: OnceCell<gdk::Texture>,

        pub(super) gradient_animation: OnceCell<adw::Animation>,
        pub(super) progress: Cell<f32>,
        pub(super) phase: Cell<u32>,

        pub(super) dark: Cell<bool>,
        pub(super) pattern_intensity: Cell<f32>,
        pub(super) pattern_is_inverted: Cell<bool>,

        pub(super) bg_colors: RefCell<Vec<graphene::Vec3>>,
        pub(super) message_colors: RefCell<Vec<graphene::Vec3>>,

        #[property(get, set = Self::setup_provider)]
        pub(super) controls_accent: Cell<bool>,
        pub(super) css_provider: OnceCell<gtk::CssProvider>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Background {
        const NAME: &'static str = "ContentBackground";
        type Type = super::Background;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for Background {
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

            let obj = self.obj();

            let settings = gio::Settings::new(crate::config::APP_ID);
            let settings_handler = settings.connect_changed(
                Some("theme-name"),
                clone!(@weak obj => move |_, _| {
                    let imp = obj.imp();

                    if imp.chat_theme.borrow().is_none() {
                        obj.refresh_theme(imp.dark.get());
                    }
                }),
            );

            self.settings.set(settings).unwrap();
            self.settings_handler.replace(Some(settings_handler));

            self.pattern.set(DEFAULT_PATTERN.to_owned()).unwrap();

            // Dark / light style
            let style_manager = adw::StyleManager::default();
            self.dark.set(style_manager.is_dark());

            style_manager.connect_dark_notify(clone!(@weak obj => move |style_manager| {
                obj.refresh_theme(style_manager.is_dark());
            }));

            if style_manager.is_high_contrast() {
                obj.add_css_class("fallback");
            }

            style_manager.connect_high_contrast_notify(clone!(@weak obj => move |style_manager| {
                if style_manager.is_high_contrast() {
                    obj.add_css_class("fallback");
                } else if obj.shader().is_some() {
                    obj.remove_css_class("fallback");
                }
            }));

            // Gradient animation
            let target = adw::CallbackAnimationTarget::new(clone!(@weak obj => move |progress| {
                let imp = obj.imp();
                imp.background_cache.take();
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

            self.gradient_animation.set(animation).unwrap();
        }

        fn dispose(&self) {
            if let Some(settings) = self.settings.get() {
                settings.disconnect(self.settings_handler.take().unwrap());
            }
        }
    }

    impl WidgetImpl for Background {
        fn realize(&self) {
            self.parent_realize();
            self.obj().ensure_shader();
            self.obj().refresh_theme(self.dark.get());
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

            self.snapshot_background(snapshot);

            if let Some(child) = widget.child() {
                widget.snapshot_child(&child, snapshot);
            }
        }
    }

    impl BinImpl for Background {}

    impl Background {
        fn snapshot_background(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();

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

        fn snapshot_gradient(
            &self,
            snapshot: &gtk::Snapshot,
            bounds: &graphene::Rect,
            size_changed: bool,
        ) {
            if self.progress.get() == 0.0 {
                let texture = match self.background_cache.take() {
                    Some(texture) if !size_changed => texture,
                    _ => {
                        let renderer = self.obj().native().unwrap().renderer();

                        renderer.render_texture(self.obj().bg_node(bounds, bounds), Some(bounds))
                    }
                };

                snapshot.append_texture(&texture, bounds);
                self.background_cache.replace(Some(texture));
            } else {
                snapshot.append_node(&self.obj().bg_node(bounds, bounds));
            }
        }

        fn snapshot_pattern(&self, snapshot: &gtk::Snapshot, bounds: &graphene::Rect) {
            let pattern_intensity = self.pattern_intensity.get();

            if pattern_intensity == 0.0 {
                return;
            }

            let texture = self.document_texture.borrow();

            let pattern = texture.as_ref().unwrap_or(&DEFAULT_PATTERN);

            let pattern_scale = if self.thumbnail_mode.get() {
                if texture.is_some() {
                    0.5
                } else {
                    0.2
                }
            } else {
                0.3
            };

            let pattern_bounds = graphene::Rect::new(
                0.0,
                0.0,
                pattern.width() as f32 * pattern_scale,
                pattern.height() as f32 * pattern_scale,
            );

            let mut matrix = [0.0; 16];
            let mut offset = [0.0; 4];

            if self.pattern_is_inverted.get() {
                matrix[15] = -pattern_intensity;
                offset = [0.08; 4];
                offset[3] = 1.0;
            } else {
                matrix[15] = pattern_intensity * 0.3;
            }

            let color_matrix = graphene::Matrix::from_float(matrix);
            let color_offset = graphene::Vec4::from_float(offset);

            snapshot.push_color_matrix(&color_matrix, &color_offset);
            snapshot.push_repeat(bounds, Some(&pattern_bounds));
            snapshot.append_scaled_texture(pattern, gsk::ScalingFilter::Trilinear, &pattern_bounds); // .append_texture(pattern, &pattern_bounds);
            snapshot.pop();
            snapshot.pop();
        }

        pub(super) fn fill_node(
            &self,
            bounds: &graphene::Rect,
            gradient_bounds: &graphene::Rect,
            colors: &[graphene::Vec3],
        ) -> gsk::RenderNode {
            match colors.len() {
                1 => gsk::ColorNode::new(&vec3_to_rgba(&colors[0]), bounds).upcast(),
                2 => gsk::LinearGradientNode::new(
                    bounds,
                    &gradient_bounds.top_left(),
                    &gradient_bounds.bottom_left(),
                    &[
                        gsk::ColorStop::new(0.0, vec3_to_rgba(&colors[0])),
                        gsk::ColorStop::new(1.0, vec3_to_rgba(&colors[1])),
                    ],
                )
                .upcast(),
                3 => {
                    log::error!("Three color gradients aren't supported yet");

                    let mut colors = colors.to_vec();
                    colors.push(colors[2]);

                    self.gradient_shader_node(bounds, gradient_bounds, &colors)
                        .upcast()
                }
                4 => self
                    .gradient_shader_node(bounds, gradient_bounds, colors)
                    .upcast(),
                _ => unreachable!("Unsupported color count"),
            }
        }

        pub(super) fn gradient_shader_node(
            &self,
            bounds: &graphene::Rect,
            gradient_bounds: &graphene::Rect,
            colors: &[graphene::Vec3],
        ) -> gsk::GLShaderNode {
            let Some(gradient_shader) = self.obj().shader() else {
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

        fn setup_provider(&self, controls_accent: bool) {
            self.controls_accent.set(controls_accent);

            if let Some(display) = gdk::Display::default() {
                let provider = self.css_provider.get_or_init(gtk::CssProvider::new);

                if controls_accent {
                    gtk::style_context_add_provider_for_display(&display, provider, 1000);
                } else {
                    gtk::style_context_remove_provider_for_display(&display, provider);
                }
            }
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

    pub(crate) fn set_chat_theme(&self, theme: Option<tdlib::types::ChatTheme>) {
        let old_theme = &self.imp().chat_theme;

        if *old_theme.borrow() != theme {
            old_theme.replace(theme);
            self.refresh_theme(adw::StyleManager::default().is_dark());
        }
    }

    pub(crate) fn set_theme(&self, theme: &tdlib::types::ThemeSettings) {
        let Some(background) = &theme.background else { return; };
        let imp = self.imp();

        imp.dark.set(background.is_dark);

        imp.pattern_intensity.take();

        self.update_accent(theme.accent_color, theme.outgoing_message_accent_color);

        let bg_fill = match &background.r#type {
            tdlib::enums::BackgroundType::Pattern(pattern) => {
                imp.pattern_is_inverted.set(pattern.is_inverted);
                imp.pattern_intensity.set(pattern.intensity as f32 / 100.0);
                &pattern.fill
            }
            tdlib::enums::BackgroundType::Fill(fill) => &fill.fill,
            tdlib::enums::BackgroundType::Wallpaper(_) => {
                unimplemented!("Wallpaper chat background")
            }
        };

        imp.document.replace(background.document.clone());

        imp.bg_colors.replace(fill_colors(bg_fill));
        imp.message_colors
            .replace(fill_colors(&theme.outgoing_message_fill));

        imp.background_cache.take();

        imp.document_texture.take();

        match &background.document {
            Some(doc) if doc.document.local.is_downloading_completed => {
                // load_document will call queue_draw fast enough
            }
            _ => self.queue_draw(),
        }

        self.load_document();
    }

    pub(crate) fn animate(&self) {
        let nothing_to_animate = self.imp().bg_colors.borrow().len() <= 2
            && self.imp().message_colors.borrow().len() <= 2;

        if nothing_to_animate {
            return;
        }

        let animation = self.imp().gradient_animation.get().unwrap();

        let val = animation.value();
        if val == 0.0 || val == 1.0 {
            animation.play()
        }
    }

    pub fn subscribe_to_redraw(&self, child: &gtk::Widget) {
        let animation = self.imp().gradient_animation.get().unwrap();
        animation.connect_value_notify(clone!(@weak child => move |_| child.queue_draw()));
    }

    pub fn bg_node(
        &self,
        bounds: &graphene::Rect,
        gradient_bounds: &graphene::Rect,
    ) -> gsk::RenderNode {
        self.imp()
            .fill_node(bounds, gradient_bounds, &self.imp().bg_colors.borrow())
    }

    pub fn message_bg_node(
        &self,
        bounds: &graphene::Rect,
        gradient_bounds: &graphene::Rect,
    ) -> gsk::RenderNode {
        self.imp()
            .fill_node(bounds, gradient_bounds, &self.imp().message_colors.borrow())
    }

    fn shader(&self) -> Option<&'static gsk::GLShader> {
        unsafe { SHADER.as_ref() }
    }

    fn ensure_shader(&self) {
        if self.shader().is_none() {
            let renderer = self.native().unwrap().renderer();

            let shader = gsk::GLShader::from_bytes(&GRADIENT_SHADER.into());
            match shader.compile(&renderer) {
                Err(e) => {
                    if !e.matches(gio::IOErrorEnum::NotSupported) {
                        log::error!("can't compile shader for gradient background {e}");
                    }
                    self.add_css_class("fallback");
                }
                Ok(_) => unsafe {
                    SHADER = Some(shader);
                },
            }
        };
    }

    pub(crate) fn set_session(&self, session: &crate::Session) {
        self.imp().session.replace(session.downgrade());
    }

    fn session(&self) -> Option<crate::Session> {
        self.ancestor(crate::Session::static_type())
            .and_downcast::<crate::Session>()
            .or(self.imp().session.borrow().upgrade())
    }

    fn load_document(&self) {
        let imp = self.imp();

        if let Some(document) = &*imp.document.borrow() {
            if let Some(session) = self.session() {
                let file = if !imp.thumbnail_mode.get() {
                    &document.document
                } else if let Some(thumbnail) = &document.thumbnail {
                    &thumbnail.file
                } else {
                    return;
                };

                imp.document_target_file_id.set(file.id);

                if file.local.is_downloading_completed {
                    self.set_downloaded_file(file);
                } else {
                    let file_id = file.id;

                    spawn(clone!(@weak self as obj => async move {
                        match session.download_file(file_id).await {
                            Ok(file) => {
                                obj.set_downloaded_file(&file);
                            }
                            Err(e) => log::error!("Can't download background: {e:?}")
                        }
                    }))
                }
            }
        }
    }

    fn set_downloaded_file(&self, file: &tdlib::types::File) {
        let imp = self.imp();

        assert!(file.local.is_downloading_completed);

        let file_id = file.id;

        if imp.document_target_file_id.get() != file_id {
            return;
        }

        if let Some((_, texture)) = imp
            .loaded_documents
            .borrow()
            .iter()
            .find(|(id, _)| *id == file_id)
        {
            imp.document_texture.replace(Some(texture.to_owned()));
            self.queue_draw();
            return;
        }

        let path = file.local.path.clone();

        let cache_path = path.replace(".tgv", "cache.png");

        if let Ok(texture) = gdk::Texture::from_filename(&cache_path) {
            self.imp()
                .loaded_documents
                .borrow_mut()
                .push((file_id, texture.clone()));
            self.imp().document_texture.replace(Some(texture));
            self.queue_draw();
            return;
        }

        let (sender, receiver) =
            glib::MainContext::channel::<(i32, gdk::Texture)>(glib::PRIORITY_DEFAULT);

        std::thread::spawn(move || {
            if path.ends_with(".tgv") {
                if let Ok(data) = std::fs::read(&path) {
                    let decompressor = gio::ZlibDecompressor::new(gio::ZlibCompressorFormat::Gzip);

                    let mut bytes_read = 0;

                    let mut accumulator = Vec::<u8>::new();

                    let mut buffer = [0; 262144];

                    loop {
                        let Ok((result, br, bw)) = decompressor.convert(
                            &data[bytes_read..],
                            &mut buffer,
                            gio::ConverterFlags::NONE,
                        ) else {
                            log::error!("Decompression error");
                            return;
                        };

                        accumulator.extend_from_slice(&buffer[..bw]);

                        bytes_read += br;

                        if result == gio::ConverterResult::Finished {
                            break;
                        }
                    }

                    match gdk::Texture::from_bytes(&glib::Bytes::from_owned(accumulator)) {
                        Ok(texture) => {
                            texture.save_to_png(&cache_path).unwrap();
                            sender.send((file_id, texture)).unwrap();
                        }
                        Err(e) => {
                            log::error!("Failed to create texture from decompressed data {e}")
                        }
                    }
                }
            } else {
                match gdk::Texture::from_filename(&path) {
                    Ok(texture) => sender.send((file_id, texture)).unwrap(),
                    Err(e) => log::error!("Failed to create texture from filename {e}"),
                };
            }
        });

        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return glib::Continue(false),
                move |(id, texture)| {
                    obj.imp().loaded_documents.borrow_mut().push((id, texture.clone()));
                    obj.imp().document_texture.replace(Some(texture));
                    obj.queue_draw();
                    glib::Continue(false)
                }
            ),
        );
    }

    fn refresh_theme(&self, dark: bool) {
        if let Some(chat_theme) = &*self.imp().chat_theme.borrow() {
            let theme = if dark {
                &chat_theme.dark_settings
            } else {
                &chat_theme.light_settings
            };

            self.set_theme(theme);
        } else {
            let chat_theme = self
                .session()
                .map(|s| s.default_chat_theme())
                .unwrap_or(crate::utils::default_theme());

            if dark {
                self.set_theme(&chat_theme.dark_settings);
            } else {
                self.set_theme(&chat_theme.light_settings);
            }
        }

        // For some reason tdlib tells that light theme is dark
        self.imp().dark.set(dark);

        if let Some(animation) = self.imp().gradient_animation.get() {
            animation.notify("value");
        }
    }

    fn update_accent(&self, accent: i32, outgoing_accent: i32) {
        if !self.controls_accent() {
            return;
        }

        let provider = self.imp().css_provider.get().unwrap();

        let mut css = String::new();

        if accent != -1 {
            let accent = vec3_to_rgba(&int_color_to_vec3(&accent));
            _ = write!(css, "@define-color accent_color {accent};");
            _ = write!(css, "@define-color accent_bg_color {accent};");
        }

        if !adw::StyleManager::default().is_dark() {
            let outgoing_accent = vec3_to_rgba(&int_color_to_vec3(&outgoing_accent));
            _ = write!(
                css,
                "@define-color outgoing_message_accent_color {outgoing_accent};"
            );
        }

        provider.load_from_data(&css);
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::new()
    }
}

fn fill_colors(fill: &tdlib::enums::BackgroundFill) -> Vec<graphene::Vec3> {
    match fill {
        tdlib::enums::BackgroundFill::FreeformGradient(gradient) if gradient.colors.len() == 4 => {
            gradient.colors.iter().map(int_color_to_vec3).collect()
        }
        tdlib::enums::BackgroundFill::Solid(solid) => vec![int_color_to_vec3(&solid.color)],
        tdlib::enums::BackgroundFill::Gradient(gradient) => vec![
            int_color_to_vec3(&gradient.top_color),
            int_color_to_vec3(&gradient.bottom_color),
        ],
        _ => unimplemented!("Unsupported background fill: {fill:?}"),
    }
}

fn int_color_to_vec3(color: &i32) -> graphene::Vec3 {
    let [_, r, g, b] = color.to_be_bytes();
    graphene::Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

fn vec3_to_rgba(vec3: &graphene::Vec3) -> gdk::RGBA {
    let [red, green, blue] = vec3.to_float();
    gdk::RGBA::new(red, green, blue, 1.0)
}
