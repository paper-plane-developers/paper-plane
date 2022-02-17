use adw::traits::ExpanderRowExt;
use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::config::APP_ID;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/preferences-window.ui")]
    pub struct PreferencesWindow {
        #[template_child]
        pub follow_system_colors_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub dark_theme_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub chat_width_spin_btn: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub custom_chat_width_exp_row: TemplateChild<adw::ExpanderRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "PreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // If the system supports color schemes, load the 'Follow system colors'
            // switch state, otherwise make that switch insensitive
            let style_manager = adw::StyleManager::default();
            if style_manager.system_supports_color_schemes() {
                let settings = gio::Settings::new(APP_ID);
                let follow_system_colors = settings.string("color-scheme") == "default";
                self.follow_system_colors_switch
                    .set_active(follow_system_colors);
            } else {
                self.follow_system_colors_switch.set_sensitive(false);
            }

            obj.setup_bindings();
        }
    }

    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
}

glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl Default for PreferencesWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferencesWindow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create PreferencesWindow")
    }

    fn setup_bindings(&self) {
        let imp = self.imp();

        // 'Follow system colors' switch state handling
        imp.follow_system_colors_switch
            .connect_active_notify(|switch| {
                let style_manager = adw::StyleManager::default();
                let settings = gio::Settings::new(APP_ID);
                if switch.is_active() {
                    // Prefer light theme unless the system prefers dark colors
                    style_manager.set_color_scheme(adw::ColorScheme::PreferLight);
                    settings.set_string("color-scheme", "default").unwrap();
                } else {
                    // Set default state for the dark theme switch
                    style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
                    settings.set_string("color-scheme", "light").unwrap();
                }
            });

        // 'Dark theme' switch state handling
        let follow_system_colors_switch = &*imp.follow_system_colors_switch;
        imp.dark_theme_switch.connect_active_notify(
            clone!(@weak follow_system_colors_switch => move |switch| {
                if !follow_system_colors_switch.is_active() {
                    let style_manager = adw::StyleManager::default();
                    let settings = gio::Settings::new(APP_ID);
                    if switch.is_active() {
                        // Dark mode
                        style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
                        settings.set_string("color-scheme", "dark").unwrap();
                    } else {
                        // Light mode
                        style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
                        settings.set_string("color-scheme", "light").unwrap();
                    }
                }
            }),
        );

        // Make the 'Dark theme' switch insensitive if the 'Follow system colors'
        // switch is active
        imp.follow_system_colors_switch
            .bind_property("active", &*imp.dark_theme_switch, "sensitive")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::INVERT_BOOLEAN)
            .build();

        // Have the 'Dark theme' switch state always updated with the dark state
        let style_manager = adw::StyleManager::default();
        style_manager
            .bind_property("dark", &*imp.dark_theme_switch, "active")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        // Initial chat width setup
        let settings = gio::Settings::new(APP_ID);
        imp.custom_chat_width_exp_row
            .set_enable_expansion(settings.boolean("use-custom-chat-width"));
        imp.chat_width_spin_btn
            .set_value(settings.int("custom-chat-width") as f64);

        // 'Use custom chat width' ExpanderRow state handling
        imp.custom_chat_width_exp_row
            .connect_enable_expansion_notify(|exp_row| {
                let settings = gio::Settings::new(APP_ID);
                if exp_row.enables_expansion() {
                    settings.set_boolean("use-custom-chat-width", true).unwrap();
                } else {
                    settings
                        .set_boolean("use-custom-chat-width", false)
                        .unwrap();
                }
            });

        // Saving changed 'custom chat width' value
        imp.chat_width_spin_btn.connect_value_changed(|spin| {
            gio::Settings::new(APP_ID)
                .set_int("custom-chat-width", spin.value_as_int())
                .unwrap();
        });
    }
}
