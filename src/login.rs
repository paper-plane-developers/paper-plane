use gettextrs::gettext;
use gtk::{
    gdk,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use locale_config::Locale;
use tdgrand::{enums::AuthorizationState, functions, types};

use crate::config;
use crate::utils::{do_async, parse_formatted_text};
use crate::DATA_DIR;

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use glib::subclass::Signal;
    use gtk::{gio, CompositeTemplate};
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/login.ui")]
    pub struct Login {
        pub client_id: Cell<i32>,
        pub tos_text: RefCell<String>,
        pub show_tos_popup: Cell<bool>,
        pub has_recovery_email_address: Cell<bool>,
        pub password_recovery_expired: Cell<bool>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub previous_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub next_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub next_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub next_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub content: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub phone_number_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub phone_number_use_qr_code_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub welcome_page_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub use_test_dc_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub qr_code_bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub qr_code_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub code_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub code_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub registration_first_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub registration_last_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub registration_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tos_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::PasswordEntry>,
        #[template_child]
        pub password_hint_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub password_hint_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_recovery_code_send_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub password_send_code_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub account_deletion_description_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_recovery_status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub password_recovery_code_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub password_recovery_error_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("login.previous", None, move |widget, _, _| {
                widget.previous()
            });
            klass.install_action("login.next", None, move |widget, _, _| widget.next());
            klass.install_action("login.use-qr-code", None, move |widget, _, _| {
                widget.request_qr_code();
            });
            klass.install_action(
                "login.go-to-forgot-password-page",
                None,
                move |widget, _, _| {
                    widget.navigate_to_page::<gtk::Editable, _, gtk::Widget>(
                        "password-forgot-page",
                        [],
                        None,
                        None,
                    );
                },
            );
            klass.install_action("login.recover-password", None, move |widget, _, _| {
                widget.recover_password();
            });
            klass.install_action(
                "login.show-no-email-access-dialog",
                None,
                move |widget, _, _| {
                    widget.show_no_email_access_dialog();
                },
            );
            klass.install_action(
                "login.show-delete-account-dialog",
                None,
                move |widget, _, _| {
                    widget.show_delete_account_dialog();
                },
            );
            klass.install_action("login.show-tos-dialog", None, move |widget, _, _| {
                widget.show_tos_dialog(false)
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("new-session", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // On each page change, decide which button to hide/show and which actions to
            // (de)activate.
            let self_ = imp::Login::from_instance(obj);
            self_
                .content
                .connect_visible_child_name_notify(clone!(@weak obj => move |_| {
                    obj.update_actions_for_visible_page()
                }));

            // Bind the use-test-dc setting to the relative switch
            let use_test_dc_switch = &*self_.use_test_dc_switch;
            let settings = gio::Settings::new(config::APP_ID);
            settings
                .bind("use-test-dc", use_test_dc_switch, "state")
                .build();

            self_.tos_label.connect_activate_link(|label, _| {
                label.activate_action("login.show-tos-dialog", None);
                gtk::Inhibit(true)
            });

            // Disable all actions by default.
            obj.disable_actions();
        }
    }

    impl WidgetImpl for Login {}
    impl BinImpl for Login {}
}

glib::wrapper! {
    pub struct Login(ObjectSubclass<imp::Login>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Login {
    fn default() -> Self {
        Self::new()
    }
}

impl Login {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Login")
    }

    pub fn login_client(&self, client_id: i32) {
        let self_ = imp::Login::from_instance(self);
        self_.client_id.set(client_id);

        // We don't know what login page to show at this point, so we show an empty page until we
        // receive an AuthenticationState that will eventually show the related login page.
        self_.main_stack.set_visible_child_name("empty-page");

        self_.phone_number_entry.set_text("");
        self_.registration_first_name_entry.set_text("");
        self_.registration_last_name_entry.set_text("");
        self_.code_entry.set_text("");
        self_.password_entry.set_text("");
    }

    pub fn set_authorization_state(&self, state: AuthorizationState) {
        let self_ = imp::Login::from_instance(self);

        match state {
            AuthorizationState::WaitTdlibParameters => {
                self.send_tdlib_parameters();
            }
            AuthorizationState::WaitEncryptionKey(_) => {
                self.send_encryption_key();
            }
            AuthorizationState::WaitPhoneNumber => {
                // The page 'phone-number-page' is the first page and thus the visible page by
                // default. This means that no transition will happen when we receive
                // 'WaitPhoneNumber'. In this case, we have to update the actions manually.
                if self_.content.visible_child_name().unwrap() == "phone-number-page" {
                    self.update_actions_for_visible_page();
                }

                // Hide the spinner before entering 'phone-number-page'.
                self_
                    .phone_number_use_qr_code_stack
                    .set_visible_child_name("image");

                self.navigate_to_page(
                    "phone-number-page",
                    [&*self_.phone_number_entry],
                    Some(&self_.welcome_page_error_label),
                    Some(&*self_.phone_number_entry),
                );
            }
            AuthorizationState::WaitCode(_) => {
                self.navigate_to_page(
                    "code-page",
                    [&*self_.code_entry],
                    Some(&self_.code_error_label),
                    Some(&*self_.code_entry),
                );
            }
            AuthorizationState::WaitOtherDeviceConfirmation(data) => {
                let size = 192;
                let bytes_per_pixel = 3;

                let data_luma = qrcode_generator::to_image_from_str(
                    data.link,
                    qrcode_generator::QrCodeEcc::Low,
                    size,
                )
                .unwrap();

                let bytes = glib::Bytes::from_owned(
                    // gdk::Texture only knows 3 byte color spaces, thus convert Luma.
                    data_luma
                        .into_iter()
                        .flat_map(|p| (0..bytes_per_pixel).map(move |_| p))
                        .collect::<Vec<_>>(),
                );

                self_
                    .qr_code_image
                    .set_paintable(Some(&gdk::MemoryTexture::new(
                        size as i32,
                        size as i32,
                        gdk::MemoryFormat::R8g8b8,
                        &bytes,
                        size * bytes_per_pixel,
                    )));

                self_.qr_code_bin.set_visible(true);

                self.navigate_to_page::<gtk::Editable, _, gtk::Widget>(
                    "qr-code-page",
                    [],
                    None,
                    None,
                );
            }
            AuthorizationState::WaitRegistration(data) => {
                self_.show_tos_popup.set(data.terms_of_service.show_popup);
                self_
                    .tos_text
                    .replace(parse_formatted_text(data.terms_of_service.text));

                self.navigate_to_page(
                    "registration-page",
                    [
                        &*self_.registration_first_name_entry,
                        &*self_.registration_last_name_entry,
                    ],
                    Some(&self_.registration_error_label),
                    Some(&*self_.registration_first_name_entry),
                );
            }
            AuthorizationState::WaitPassword(data) => {
                // If we do RequestAuthenticationPasswordRecovery we will land in this arm again.
                // To avoid transition back, clearing the entries and to save cpu time, we check
                // whether we are in the password-forgot-page.
                if self_.content.visible_child_name().unwrap() == "password-forgot-page" {
                    return;
                }

                // When we enter the password page, the password to be entered should be masked by
                // default, so the peek icon is turned off and on again.
                self_.password_entry.set_show_peek_icon(false);
                self_.password_entry.set_show_peek_icon(true);

                self_
                    .password_hint_action_row
                    .set_visible(!data.password_hint.is_empty());
                self_.password_hint_label.set_text(&data.password_hint);

                let account_deletion_preface = if data.has_recovery_email_address {
                    self_
                        .password_recovery_status_page
                        .set_description(Some(&gettext!(
                            "The code was sent to {}.",
                            data.recovery_email_address_pattern
                        )));
                    gettext(
                            "One way to continue using your account is to delete your account and then recreate it"
                        )
                } else {
                    self_.password_recovery_status_page.set_description(None);
                    gettext(
                        "Since you have not provided a recovery email address, the only way to continue using your account is to delete your account and then recreate it"
                    )
                };

                self_.account_deletion_description_label.set_label(&format!(
                    "{}. {}",
                    account_deletion_preface,
                    gettext(
                        "Please note, you will lose all your chats and messages, along with any media and files you shared!"
                    )
                ));
                self_
                    .password_recovery_code_send_box
                    .set_visible(data.has_recovery_email_address);
                self_
                    .has_recovery_email_address
                    .set(data.has_recovery_email_address);

                // When we first enter WaitPassword, we assume that the mail with the recovery
                // code hasn't been sent, yet.
                self_.password_recovery_expired.set(true);

                self.navigate_to_page(
                    "password-page",
                    [&*self_.password_entry],
                    Some(&self_.password_error_label),
                    Some(&*self_.password_entry),
                );
            }
            AuthorizationState::Ready => {
                self.disable_actions();
                self_.qr_code_bin.set_visible(false);
                // Clear the qr code image save some potential memory.
                self_
                    .qr_code_image
                    .set_paintable(None as Option<&gdk::Paintable>);
                self.emit_by_name("new-session", &[]).unwrap();
            }
            _ => {}
        }
    }

    fn navigate_to_page<'a, E, I, W>(
        &self,
        page_name: &str,
        editables_to_clear: I,
        error_label_to_clear: Option<&gtk::Label>,
        widget_to_focus: Option<&W>,
    ) where
        E: IsA<gtk::Editable>,
        I: IntoIterator<Item = &'a E>,
        W: IsA<gtk::Widget>,
    {
        let self_ = imp::Login::from_instance(self);

        // Before transition to the page, be sure to reset the error label because it still might
        // conatain an error message from the time when it was previously visited.
        if let Some(error_label_to_clear) = error_label_to_clear {
            error_label_to_clear.set_label("");
        }
        // Also clear all editables on that page.
        editables_to_clear
            .into_iter()
            .for_each(|editable| editable.set_text(""));

        self_.content.set_visible_child_name(page_name);

        // After we've transitioned to a new login page, let's be sure that we set the stack here
        // to an ancestor widget of the login leaflet because we might still be in the empty page.
        self_.main_stack.set_visible_child_name("login-flow-page");

        self.unfreeze();
        if let Some(widget_to_focus) = widget_to_focus {
            widget_to_focus.grab_focus();
        }
    }

    fn update_actions_for_visible_page(&self) {
        let self_ = imp::Login::from_instance(self);

        let visible_page = self_.content.visible_child_name().unwrap();

        let is_previous_valid = visible_page.as_str() != "phone-number-page";
        let is_next_valid = visible_page.as_str() != "password-forgot-page"
            && visible_page.as_str() != "qr-code-page";

        self_.previous_button.set_visible(is_previous_valid);
        self_.next_button.set_visible(is_next_valid);

        self.action_set_enabled("login.previous", is_previous_valid);
        self.action_set_enabled("login.next", is_next_valid);
        self.action_set_enabled("login.use-qr-code", visible_page == "phone-number-page");
        self.action_set_enabled(
            "login.go-to-forgot-password-page",
            visible_page == "password-page",
        );
        self.action_set_enabled(
            "login.recover-password",
            visible_page == "password-forgot-page" && self_.has_recovery_email_address.get(),
        );
        self.action_set_enabled(
            "login.show-no-email-access-dialog",
            visible_page == "password-recovery-page",
        );
        self.action_set_enabled(
            "login.show-delete-account-dialog",
            visible_page == "password-forgot-page",
        );
        self.action_set_enabled("login.show-tos-dialog", visible_page == "registration-page");
    }

    fn previous(&self) {
        let self_ = imp::Login::from_instance(self);

        match self_.content.visible_child_name().unwrap().as_str() {
            "qr-code-page" => self.leave_qr_code_page(),
            "password-forgot-page" => self.navigate_to_page::<gtk::Editable, _, _>(
                "password-page",
                [],
                None,
                Some(&*self_.password_entry),
            ),
            "password-recovery-page" => self.navigate_to_page::<gtk::Editable, _, gtk::Widget>(
                "password-forgot-page",
                [],
                None,
                None,
            ),
            _ => self.navigate_to_page::<gtk::Editable, _, _>(
                "phone-number-page",
                [],
                None,
                Some(&*self_.phone_number_entry),
            ),
        }
    }

    fn next(&self) {
        self.freeze_with_next_spinner();

        let self_ = imp::Login::from_instance(self);
        let visible_page = self_.content.visible_child_name().unwrap();

        match visible_page.as_str() {
            "phone-number-page" => self.send_phone_number(),
            "code-page" => self.send_code(),
            "registration-page" => {
                if self_.show_tos_popup.get() {
                    // Force the ToS dialog for the user before he can proceed
                    self.show_tos_dialog(true);
                } else {
                    // Just proceed if the user either doesn't need to accept the ToS
                    self.send_registration()
                }
            }
            "password-page" => self.send_password(),
            "password-recovery-page" => self.send_password_recovery_code(),
            other => unreachable!("no page named '{}'", other),
        }
    }

    fn request_qr_code(&self) {
        self.freeze();
        imp::Login::from_instance(self)
            .phone_number_use_qr_code_stack
            .set_visible_child_name("spinner");

        let client_id = self.client_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::RequestQrCodeAuthentication::new()
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                let self_ = imp::Login::from_instance(&obj);
                obj.handle_user_result(
                    result,
                    &self_.welcome_page_error_label,
                    &*self_.phone_number_entry
                );
            }),
        );
    }

    fn leave_qr_code_page(&self) {
        self.freeze_with_previous_spinner();

        let self_ = imp::Login::from_instance(self);
        self_.qr_code_bin.set_visible(false);

        let client_id = self.client_id();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                // We actually need to logout to stop tdlib sending us new links.
                // https://github.com/tdlib/td/issues/1645
                functions::LogOut::new().send(client_id).await
            },
            clone!(@weak self as obj => move |result| async move {
                let self_ = imp::Login::from_instance(&obj);
                if result.is_err() {
                    self_.qr_code_bin.set_visible(true);
                    obj.unfreeze();
                    // TODO: We also need to handle potential errors here and inform the user that
                    // the change to phone number identification failed (Toast?).
                }
            }),
        );
    }

    fn show_tos_dialog(&self, user_needs_to_accept: bool) {
        let self_ = imp::Login::from_instance(self);

        let builder = gtk::MessageDialog::builder()
            .use_markup(true)
            .secondary_text(&*self_.tos_text.borrow())
            .modal(true)
            .transient_for(self.root().unwrap().downcast_ref::<gtk::Window>().unwrap());

        let dialog = if user_needs_to_accept {
            builder
                .buttons(gtk::ButtonsType::YesNo)
                .text(&gettext("Do You Accept the Terms of Service?"))
        } else {
            builder
                .buttons(gtk::ButtonsType::Ok)
                .text(&gettext("Terms of Service"))
        }
        .build();

        dialog.run_async(clone!(@weak self as obj => move |dialog, response| {
            if matches!(response, gtk::ResponseType::No) {
                // If the user declines the ToS, don't proceed and just stay in
                // the view but unfreeze it again.
                obj.unfreeze();
            } else if matches!(response, gtk::ResponseType::Yes) {
                // User has accepted the ToS, so we can proceed in the login
                // flow.
                obj.send_registration();
            }
            dialog.close();
        }));
    }

    fn disable_actions(&self) {
        self.action_set_enabled("login.previous", false);
        self.action_set_enabled("login.next", false);
        self.action_set_enabled("login.use-qr-code", false);
        self.action_set_enabled("login.go-to-forgot-password-page", false);
        self.action_set_enabled("login.recover-password", false);
        self.action_set_enabled("login.show-no-email-access-dialog", false);
        self.action_set_enabled("login.show-delete-account-dialog", false);
        self.action_set_enabled("login.show-tos-dialog", false);
    }

    fn freeze(&self) {
        self.disable_actions();
        imp::Login::from_instance(self).content.set_sensitive(false);
    }

    fn freeze_with_previous_spinner(&self) {
        self.freeze();

        let self_ = imp::Login::from_instance(self);
        self_.previous_stack.set_visible_child_name("spinner");
    }

    fn freeze_with_next_spinner(&self) {
        self.freeze();

        let self_ = imp::Login::from_instance(self);
        self_
            .next_stack
            .set_visible_child(&self_.next_spinner.get());
    }

    fn unfreeze(&self) {
        let self_ = imp::Login::from_instance(self);
        self_.previous_stack.set_visible_child_name("text");
        self_.next_stack.set_visible_child(&self_.next_label.get());
        self_.content.set_sensitive(true);
    }

    fn send_tdlib_parameters(&self) {
        let self_ = imp::Login::from_instance(self);
        let client_id = self_.client_id.get();
        let use_test_dc = self_.use_test_dc_switch.state();

        let database_directory = DATA_DIR
            .get()
            .unwrap()
            .to_str()
            .expect("Data directory path is not a valid unicode string")
            .to_owned();

        let system_language_code = {
            let locale = Locale::current().to_string();
            if !locale.is_empty() {
                locale
            } else {
                "en_US".to_string()
            }
        };
        let parameters = types::TdlibParameters {
            use_test_dc,
            database_directory,
            use_message_database: true,
            use_secret_chats: true,
            api_id: config::TG_API_ID,
            api_hash: config::TG_API_HASH.to_string(),
            system_language_code,
            device_model: "Desktop".to_string(),
            application_version: config::VERSION.to_string(),
            enable_storage_optimizer: true,
            ..types::TdlibParameters::default()
        };
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::SetTdlibParameters::new()
                    .parameters(parameters)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    show_error_label(
                        &imp::Login::from_instance(&obj).welcome_page_error_label,
                        &err.message
                    );
                }
            }),
        );
    }

    fn send_encryption_key(&self) {
        let self_ = imp::Login::from_instance(self);
        let client_id = self_.client_id.get();
        let encryption_key = "".to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::CheckDatabaseEncryptionKey::new()
                    .encryption_key(encryption_key)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                if let Err(err) = result {
                    show_error_label(
                        &imp::Login::from_instance(&obj).welcome_page_error_label,
                        &err.message
                    )
                }
            }),
        );
    }

    fn send_phone_number(&self) {
        let self_ = imp::Login::from_instance(self);

        reset_error_label(&self_.welcome_page_error_label);

        let client_id = self_.client_id.get();
        let phone_number = self_.phone_number_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::SetAuthenticationPhoneNumber::new()
                    .phone_number(phone_number)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                let self_ = imp::Login::from_instance(&obj);
                obj.handle_user_result(
                    result,
                    &self_.welcome_page_error_label,
                    &*self_.phone_number_entry
                );
            }),
        );
    }

    fn send_code(&self) {
        let self_ = imp::Login::from_instance(self);

        reset_error_label(&self_.code_error_label);

        let client_id = self_.client_id.get();
        let code = self_.code_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::CheckAuthenticationCode::new()
                    .code(code)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                let self_ = imp::Login::from_instance(&obj);
                obj.handle_user_result(result, &self_.code_error_label, &*self_.code_entry);
            }),
        );
    }

    fn send_registration(&self) {
        let self_ = imp::Login::from_instance(self);

        reset_error_label(&self_.registration_error_label);

        let client_id = self_.client_id.get();
        let first_name = self_.registration_first_name_entry.text().to_string();
        let last_name = self_.registration_last_name_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::RegisterUser::new()
                    .first_name(first_name)
                    .last_name(last_name)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                let self_ = imp::Login::from_instance(&obj);
                obj.handle_user_result(
                    result,
                    &self_.registration_error_label,
                    &*self_.registration_first_name_entry
                );
            }),
        );
    }

    fn send_password(&self) {
        let self_ = imp::Login::from_instance(self);

        reset_error_label(&self_.password_error_label);

        let client_id = self_.client_id.get();
        let password = self_.password_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::CheckAuthenticationPassword::new()
                    .password(password)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                let self_ = imp::Login::from_instance(&obj);
                obj.handle_user_result(
                    result,
                    &self_.password_error_label,
                    &*self_.password_entry
                );
            }),
        );
    }

    fn recover_password(&self) {
        let self_ = imp::Login::from_instance(self);

        if self_.password_recovery_expired.get() {
            // We need to tell tdlib to send us the recovery code via mail (again).
            self.freeze();
            self_
                .password_send_code_stack
                .set_visible_child_name("spinner");

            let client_id = imp::Login::from_instance(self).client_id.get();
            do_async(
                glib::PRIORITY_DEFAULT_IDLE,
                async move {
                    functions::RequestAuthenticationPasswordRecovery::new()
                        .send(client_id)
                        .await
                },
                clone!(@weak self as obj => move |result| async move {
                    let self_ = imp::Login::from_instance(&obj);

                    // Remove the spinner from the button.
                    self_
                        .password_send_code_stack
                        .set_visible_child_name("image");

                    if result.is_ok() {
                        // Save that we do not need to resend the mail when we enter the recovery
                        // page the next time.
                        self_.password_recovery_expired.set(false);
                        obj.navigate_to_page(
                            "password-recovery-page",
                            [&*self_.password_recovery_code_entry],
                            Some(&self_.password_recovery_error_label),
                            Some(&*self_.password_recovery_code_entry),
                        );
                    } else {
                        obj.update_actions_for_visible_page();
                        // TODO: We also need to handle potiential errors here and inform the user.
                    }

                    obj.unfreeze();
                }),
            );
        } else {
            // The code has been send already via mail.
            self.navigate_to_page(
                "password-recovery-page",
                [&*self_.password_recovery_code_entry],
                Some(&self_.password_recovery_error_label),
                Some(&*self_.password_recovery_code_entry),
            );
        }
    }

    fn show_delete_account_dialog(&self) {
        let dialog = gtk::MessageDialog::builder()
            .text(&gettext("Warning"))
            .secondary_text(&gettext(
                "You will lose all your chats and messages, along with any media and files you shared!\n\nDo you want to delete your account?",
            ))
            .buttons(gtk::ButtonsType::Cancel)
            .modal(true)
            .transient_for(self.root().unwrap().downcast_ref::<gtk::Window>().unwrap())
            .build();

        dialog.add_action_widget(
            &gtk::Button::builder()
                .use_underline(true)
                .label("_Delete Account")
                .css_classes(vec!["destructive-action".to_string()])
                .build(),
            gtk::ResponseType::Accept,
        );

        dialog.run_async(clone!(@weak self as obj => move |dialog, response_id| {
            dialog.close();

            if matches!(response_id, gtk::ResponseType::Accept) {
                obj.freeze();
                let client_id = imp::Login::from_instance(&obj).client_id.get();
                do_async(
                    glib::PRIORITY_DEFAULT_IDLE,
                    async move {
                        functions::DeleteAccount::new()
                            .reason(String::from("cloud password lost and not recoverable"))
                            .send(client_id)
                            .await
                    },
                    clone!(@weak obj => move |result| async move {
                        // Just unfreeze in case of an error, else stay frozen until we are
                        // redirected to the welcome page.
                        if result.is_err() {
                            obj.update_actions_for_visible_page();
                            obj.unfreeze();
                            // TODO: We also need to handle potiential errors here and inform the
                            // user.
                        }
                    }),
                );
            } else {
                imp::Login::from_instance(&obj)
                    .password_entry
                    .grab_focus();
            }
        }));
    }

    fn send_password_recovery_code(&self) {
        let self_ = imp::Login::from_instance(self);
        let client_id = self_.client_id.get();
        let recovery_code = self_.password_recovery_code_entry.text().to_string();
        do_async(
            glib::PRIORITY_DEFAULT_IDLE,
            async move {
                functions::RecoverAuthenticationPassword::new()
                    .recovery_code(recovery_code)
                    .send(client_id)
                    .await
            },
            clone!(@weak self as obj => move |result| async move {
                let self_ = imp::Login::from_instance(&obj);

                if let Err(err) = result {
                    if err.message == "PASSWORD_RECOVERY_EXPIRED" {
                        // The same procedure is used as for the official client (as far as I
                        // understood from the code). Alternatively, we could send the user a new
                        // code, indicate that and stay on the recovery page.
                        self_.password_recovery_expired.set(true);
                        obj.navigate_to_page::<gtk::Editable, _, _>(
                            "password-page", [],
                            None,
                            Some(&*self_.password_entry)
                        );
                    } else {
                        obj.handle_user_error(
                            &err,
                            &self_.password_recovery_error_label,
                            &*self_.password_recovery_code_entry
                        );
                    }
                }
            }),
        );
    }

    fn show_no_email_access_dialog(&self) {
        let dialog = gtk::MessageDialog::builder()
            .text(&gettext("Sorry"))
            .secondary_text(&gettext(
                "If you can't restore access to the email, your remaining options are either to remember your password or to delete and recreate your account.",
            ))
            .buttons(gtk::ButtonsType::Close)
            .modal(true)
            .transient_for(self.root().unwrap().downcast_ref::<gtk::Window>().unwrap())
            .build();

        dialog.add_button(&gettext("_Go Back"), gtk::ResponseType::Other(0));

        dialog.run_async(clone!(@weak self as obj => move |dialog, response_id| {
            dialog.close();

            if let gtk::ResponseType::Other(_) = response_id {
                obj.navigate_to_page::<gtk::Editable, _, gtk::Widget>(
                    "password-forgot-page",
                    [],
                    None,
                    None,
                );
            } else {
                imp::Login::from_instance(&obj)
                    .password_recovery_code_entry
                    .grab_focus();
            }
        }));
    }

    fn handle_user_result<T, W: IsA<gtk::Widget>>(
        &self,
        result: Result<T, types::Error>,
        error_label: &gtk::Label,
        widget_to_focus: &W,
    ) -> Option<T> {
        match result {
            Err(err) => {
                self.handle_user_error(&err, error_label, widget_to_focus);
                None
            }
            Ok(t) => Some(t),
        }
    }

    fn handle_user_error<W: IsA<gtk::Widget>>(
        &self,
        err: &types::Error,
        error_label: &gtk::Label,
        widget_to_focus: &W,
    ) {
        show_error_label(error_label, &err.message);
        // In case of an error we do not switch pages. So invalidate actions here.
        self.update_actions_for_visible_page();
        self.unfreeze();
        // Grab focus for entry again after error.
        widget_to_focus.grab_focus();
    }

    pub fn client_id(&self) -> i32 {
        let self_ = imp::Login::from_instance(self);
        self_.client_id.get()
    }

    pub fn connect_new_session<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("new-session", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);

            None
        })
        .unwrap()
    }
}

fn show_error_label(error_label: &gtk::Label, message: &str) {
    error_label.set_text(message);
    error_label.set_visible(true);
}

fn reset_error_label(error_label: &gtk::Label) {
    error_label.set_text("");
    error_label.set_visible(false);
}
