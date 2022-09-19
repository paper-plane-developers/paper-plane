use adw::prelude::*;
use gettextrs::gettext;
use gtk::gdk;
use gtk::glib::{self, clone};
use gtk::subclass::prelude::*;
use tdlib::enums::{self, AuthenticationCodeType, AuthorizationState};
use tdlib::{functions, types};

use crate::session::Session;
use crate::session_manager::SessionManager;
use crate::tdlib::CountryList;
use crate::utils::{log_out, parse_formatted_text, send_tdlib_parameters, spawn};

mod imp {
    use super::*;
    use adw::subclass::prelude::BinImpl;
    use gtk::glib::SourceId;
    use gtk::CompositeTemplate;
    use once_cell::sync::OnceCell;
    use std::cell::{Cell, RefCell};

    use crate::phone_number_input::PhoneNumberInput;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/login.ui")]
    pub(crate) struct Login {
        pub(super) session_manager: OnceCell<SessionManager>,
        pub(super) client_id: Cell<i32>,
        pub(super) session: RefCell<Option<Session>>,
        pub(super) countries_retrieved: Cell<bool>,
        pub(super) tos_text: RefCell<String>,
        pub(super) show_tos_popup: Cell<bool>,
        pub(super) has_recovery_email_address: Cell<bool>,
        pub(super) password_recovery_expired: Cell<bool>,
        pub(super) code_has_next_type: Cell<bool>,
        pub(super) code_next_type_countdown_id: RefCell<Option<SourceId>>,
        #[template_child]
        pub(super) outer_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) previous_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) next_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) next_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) next_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) next_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub(super) content: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub(super) phone_number_input: TemplateChild<PhoneNumberInput>,
        #[template_child]
        pub(super) phone_number_use_qr_code_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) welcome_page_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) qr_code_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) code_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub(super) code_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) code_resend_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) code_resend_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) code_timeout_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) code_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) registration_first_name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) registration_last_name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) registration_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) tos_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) password_entry_row: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub(super) password_hint_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) password_hint_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) password_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) password_recovery_code_send_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) password_send_code_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) account_deletion_description_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) password_recovery_status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub(super) password_recovery_code_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) password_recovery_error_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("login.previous", None, move |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    widget.previous().await;
                }));
            });
            klass.install_action("login.next", None, move |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    widget.next().await;
                }));
            });
            klass.install_action("login.use-qr-code", None, move |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    widget.request_qr_code().await;
                }));
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
            klass.install_action("login.resend-auth-code", None, move |widget, _, _| {
                spawn(clone!(@weak widget => async move {
                    widget.resend_auth_code().await;
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // On each page change, decide which button to hide/show and which actions to
            // (de)activate.
            self.content
                .connect_visible_child_name_notify(clone!(@weak obj => move |_| {
                    obj.update_actions_for_visible_page()
                }));

            self.tos_label.connect_activate_link(|label, _| {
                label
                    .activate_action("login.show-tos-dialog", None)
                    .unwrap();
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
    pub(crate) struct Login(ObjectSubclass<imp::Login>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for Login {
    fn default() -> Self {
        Self::new()
    }
}

impl Login {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Login")
    }

    pub(crate) fn set_session_manager(&self, session_manager: SessionManager) {
        self.imp().session_manager.set(session_manager).unwrap();
    }

    pub(crate) fn login_client(&self, client_id: i32, session: Session) {
        let imp = self.imp();
        imp.client_id.set(client_id);

        imp.session.replace(Some(session));

        imp.countries_retrieved.set(false);
        imp.phone_number_input.set_number("");
        imp.registration_first_name_entry_row.set_text("");
        imp.registration_last_name_entry_row.set_text("");
        imp.code_entry_row.set_text("");
        imp.password_entry_row.set_text("");
    }

    pub(crate) fn set_authorization_state(&self, state: AuthorizationState) {
        let imp = self.imp();

        match state {
            AuthorizationState::WaitTdlibParameters => {
                let client_id = imp.client_id.get();
                let database_info = imp
                    .session
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .database_info()
                    .0
                    .clone();

                spawn(clone!(@weak self as obj => async move {
                    let result = send_tdlib_parameters(client_id, &database_info).await;

                    if let Err(err) = result {
                        show_error_label(&obj.imp().welcome_page_error_label, &err.message);
                    }
                }));
            }
            AuthorizationState::WaitEncryptionKey(_) => {
                spawn(clone!(@weak self as obj => async move {
                    obj.send_encryption_key().await;
                }));
            }
            AuthorizationState::WaitPhoneNumber => {
                if !imp.countries_retrieved.get() {
                    imp.countries_retrieved.set(true);

                    let use_test_dc = self.use_test_dc();
                    spawn(clone!(@weak self as obj => async move {
                        let imp = obj.imp();
                        match functions::get_countries(imp.client_id.get()).await {
                            Ok(enums::Countries::Countries(countries)) => {
                                imp.phone_number_input.set_model(
                                    Some(&CountryList::from_td_object(countries, use_test_dc))
                                );
                                imp.phone_number_input.select_number_without_calling_code();
                            }
                            Err(_) => {
                                imp.phone_number_input.set_model(None);
                                // TODO: Show a toast notification.
                            }
                        }
                    }));
                }

                // The page 'phone-number-page' is the first page and thus the visible page by
                // default. This means that no transition will happen when we receive
                // 'WaitPhoneNumber'. In this case, we have to update the actions manually.
                if imp.content.visible_child_name().unwrap() == "phone-number-page" {
                    self.update_actions_for_visible_page();
                }

                // Hide the spinner before entering 'phone-number-page'.
                imp.phone_number_use_qr_code_stack
                    .set_visible_child_name("image");

                self.navigate_to_page(
                    "phone-number-page",
                    [&*imp.phone_number_input],
                    Some(&imp.welcome_page_error_label),
                    Some(&*imp.phone_number_input),
                );
            }
            AuthorizationState::WaitCode(data) => {
                imp.code_page.set_description(Some(&gettext!(
                    "The code will arrive to you via {}.",
                    stringify_auth_code_type(data.code_info.r#type),
                )));

                self.update_code_resend_state(data.code_info.next_type, data.code_info.timeout);

                self.navigate_to_page(
                    "code-page",
                    [&*imp.code_entry_row],
                    Some(&imp.code_error_label),
                    Some(&*imp.code_entry_row),
                );
            }
            AuthorizationState::WaitOtherDeviceConfirmation(data) => {
                let size = imp.qr_code_image.pixel_size() as usize;
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

                imp.qr_code_image
                    .set_paintable(Some(&gdk::MemoryTexture::new(
                        size as i32,
                        size as i32,
                        gdk::MemoryFormat::R8g8b8,
                        &bytes,
                        size * bytes_per_pixel,
                    )));

                self.navigate_to_page::<gtk::Editable, _, gtk::Widget>(
                    "qr-code-page",
                    [],
                    None,
                    None,
                );
            }
            AuthorizationState::WaitRegistration(data) => {
                imp.show_tos_popup.set(data.terms_of_service.show_popup);
                imp.tos_text
                    .replace(parse_formatted_text(data.terms_of_service.text));

                self.navigate_to_page(
                    "registration-page",
                    [
                        &*imp.registration_first_name_entry_row,
                        &*imp.registration_last_name_entry_row,
                    ],
                    Some(&imp.registration_error_label),
                    Some(&*imp.registration_first_name_entry_row),
                );
            }
            AuthorizationState::WaitPassword(data) => {
                // If we do RequestAuthenticationPasswordRecovery we will land in this arm again.
                // To avoid transition back, clearing the entries and to save cpu time, we check
                // whether we are in the password-forgot-page.
                if imp.content.visible_child_name().unwrap() == "password-forgot-page" {
                    return;
                }

                imp.password_hint_action_row
                    .set_visible(!data.password_hint.is_empty());
                imp.password_hint_label.set_text(&data.password_hint);

                let account_deletion_preface = if data.has_recovery_email_address {
                    imp.password_recovery_status_page
                        .set_description(Some(&gettext!(
                            "The code was sent to {}.",
                            data.recovery_email_address_pattern
                        )));
                    gettext(
                        "One way to continue using your account is to delete and then recreate it",
                    )
                } else {
                    imp.password_recovery_status_page.set_description(None);
                    gettext(
                        "Since you have not provided a recovery e-mail address, the only way to continue using your account is to delete and then recreate it"
                    )
                };

                imp.account_deletion_description_label.set_label(&format!(
                    "{}. {}",
                    account_deletion_preface,
                    gettext(
                        "Please note, you will lose all your chats and messages, along with any media and files you shared!"
                    )
                ));
                imp.password_recovery_code_send_box
                    .set_visible(data.has_recovery_email_address);
                imp.has_recovery_email_address
                    .set(data.has_recovery_email_address);

                // When we first enter WaitPassword, we assume that the mail with the recovery
                // code hasn't been sent, yet.
                imp.password_recovery_expired.set(true);

                self.navigate_to_page(
                    "password-page",
                    [&*imp.password_entry_row],
                    Some(&imp.password_error_label),
                    Some(&*imp.password_entry_row),
                );
            }
            AuthorizationState::Ready => {
                self.disable_actions();

                // Clear the qr code image save some potential memory.
                imp.qr_code_image.set_paintable(gdk::Paintable::NONE);

                spawn(clone!(@weak self as obj => async move {
                    let imp = obj.imp();
                    imp.session_manager.get().unwrap().add_logged_in_session(
                        imp.client_id.get(),
                        imp.session.take().unwrap(),
                        true,
                    ).await;
                }));

                // Make everything invisible.
                imp.outer_box.set_visible(false);
            }
            _ => {}
        }
    }

    fn update_code_resend_state(
        &self,
        auth_code_next_type: Option<AuthenticationCodeType>,
        timeout: i32,
    ) {
        // Always stop the resend countdown first.
        self.stop_code_next_type_countdown();

        let imp = self.imp();

        imp.code_has_next_type
            .replace(auth_code_next_type.is_some());

        self.action_set_enabled("login.resend-auth-code", imp.code_has_next_type.get());

        match auth_code_next_type {
            None => {
                imp.code_resend_stack.set_visible_child_name("disabled");
            }
            Some(code_type) => {
                imp.code_resend_stack
                    .set_visible_child(&*imp.code_resend_button);
                imp.code_resend_button.set_label(&gettext!(
                    "_Resend via {}",
                    &stringify_auth_code_type(code_type)
                ));

                let mut countdown = timeout;
                if countdown > 0 {
                    imp.code_timeout_label.set_visible(true);
                    imp.code_timeout_label
                        .set_label(&gettext!("Please still wait {} seconds", countdown));

                    let source_id = glib::timeout_add_seconds_local(
                        1,
                        clone!(@weak self as obj => @default-return glib::Continue(false), move || {
                            let imp = obj.imp();
                            countdown -= 1;
                            glib::Continue(if countdown == 0 {
                                obj.stop_code_next_type_countdown();
                                false
                            } else {
                                imp.code_timeout_label.set_label(&gettext!(
                                    "Please still wait {} seconds",
                                    countdown
                                ));
                                true
                            })
                        }),
                    );
                    imp.code_next_type_countdown_id.replace(Some(source_id));
                }
            }
        };
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
        let imp = self.imp();

        // Before transition to the page, be sure to reset the error label because it still might
        // contain an error message from the time when it was previously visited.
        if let Some(error_label_to_clear) = error_label_to_clear {
            error_label_to_clear.set_label("");
        }
        // Also clear all editables on that page.
        editables_to_clear
            .into_iter()
            .for_each(|editable| editable.set_text(""));

        imp.content.set_visible_child_name(page_name);

        // Make sure everything is visible.
        imp.outer_box.set_visible(true);

        self.unfreeze();
        if let Some(widget_to_focus) = widget_to_focus {
            widget_to_focus.grab_focus();
        }
    }

    fn update_actions_for_visible_page(&self) {
        let imp = self.imp();

        let visible_page = imp.content.visible_child_name().unwrap();

        let is_previous_valid = imp
            .session_manager
            .get()
            .map(|session_manager| session_manager.sessions().n_items() > 0)
            .unwrap_or_default()
            || visible_page.as_str() != "phone-number-page";

        let is_next_valid = visible_page.as_str() != "password-forgot-page"
            && visible_page.as_str() != "qr-code-page";

        imp.previous_button.set_visible(is_previous_valid);
        imp.next_button.set_visible(is_next_valid);

        self.action_set_enabled("login.previous", is_previous_valid);
        self.action_set_enabled("login.next", is_next_valid);
        self.action_set_enabled("login.use-qr-code", visible_page == "phone-number-page");
        self.action_set_enabled(
            "login.resend-auth-code",
            visible_page == "code-page" && imp.code_has_next_type.get(),
        );
        self.action_set_enabled(
            "login.go-to-forgot-password-page",
            visible_page == "password-page",
        );
        self.action_set_enabled(
            "login.recover-password",
            visible_page == "password-forgot-page" && imp.has_recovery_email_address.get(),
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

    fn stop_code_next_type_countdown(&self) {
        self.imp().code_timeout_label.set_visible(false);

        if let Some(source_id) = self.imp().code_next_type_countdown_id.take() {
            source_id.remove();
        }
    }

    async fn previous(&self) {
        let imp = self.imp();

        match imp.content.visible_child_name().unwrap().as_str() {
            "phone-number-page" => {
                self.freeze_with_previous_spinner();

                // Logout the client when login is aborted.
                log_out(imp.client_id.get()).await;
                imp.session_manager.get().unwrap().switch_to_sessions(None);
            }
            "qr-code-page" => self.leave_qr_code_page().await,
            "password-forgot-page" => self.navigate_to_page::<gtk::Editable, _, _>(
                "password-page",
                [],
                None,
                Some(&*imp.password_entry_row),
            ),
            "password-recovery-page" => self.navigate_to_page::<gtk::Editable, _, gtk::Widget>(
                "password-forgot-page",
                [],
                None,
                None,
            ),
            other => {
                if other == "code-page" {
                    self.stop_code_next_type_countdown();
                }
                self.navigate_to_page::<gtk::Editable, _, _>(
                    "phone-number-page",
                    [],
                    None,
                    Some(&*imp.phone_number_input),
                )
            }
        }
    }

    async fn next(&self) {
        self.freeze_with_next_spinner();

        let imp = self.imp();
        let visible_page = imp.content.visible_child_name().unwrap();

        match visible_page.as_str() {
            "phone-number-page" => self.send_phone_number().await,
            "code-page" => self.send_code().await,
            "registration-page" => {
                if imp.show_tos_popup.get() {
                    // Force the ToS dialog for the user before he can proceed
                    self.show_tos_dialog(true);
                } else {
                    // Just proceed if the user either doesn't need to accept the ToS
                    self.send_registration().await
                }
            }
            "password-page" => self.send_password().await,
            "password-recovery-page" => self.send_password_recovery_code().await,
            other => unreachable!("no page named '{}'", other),
        }
    }

    async fn request_qr_code(&self) {
        self.freeze();

        let imp = self.imp();
        imp.phone_number_use_qr_code_stack
            .set_visible_child_name("spinner");

        let other_user_ids = imp
            .session_manager
            .get()
            .unwrap()
            .logged_in_users()
            .into_iter()
            .map(|user| user.id())
            .collect();
        let client_id = imp.client_id.get();

        let result = functions::request_qr_code_authentication(other_user_ids, client_id).await;

        self.handle_user_result(
            result,
            &imp.welcome_page_error_label,
            &*imp.phone_number_input,
        );
    }

    async fn leave_qr_code_page(&self) {
        // We actually need to logout to stop tdlib sending us new links.
        // https://github.com/tdlib/td/issues/1645
        let imp = self.imp();

        log_out(imp.client_id.get()).await;
        imp.session_manager
            .get()
            .unwrap()
            .add_new_session(self.use_test_dc());
    }

    fn show_tos_dialog(&self, user_needs_to_accept: bool) {
        let dialog = adw::MessageDialog::builder()
            .body_use_markup(true)
            .body(&*self.imp().tos_text.borrow())
            .transient_for(self.root().unwrap().downcast_ref::<gtk::Window>().unwrap())
            .build();

        if user_needs_to_accept {
            dialog.set_heading(Some(&gettext("Do you accept the Terms of Service?")));
            dialog.add_responses(&[("no", &gettext("_No")), ("yes", &gettext("_Yes"))]);
            dialog.set_default_response(Some("no"));
        } else {
            dialog.set_heading(Some(&gettext("Terms of Service")));
            dialog.add_response("ok", &gettext("_OK"));
            dialog.set_default_response(Some("ok"));
        }

        dialog.run_async(
            None,
            clone!(@weak self as obj => move |_, response| {
                if response == "no" {
                    // If the user declines the ToS, don't proceed and just stay in
                    // the view but unfreeze it again.
                    obj.unfreeze();
                } else if response == "yes" {
                    // User has accepted the ToS, so we can proceed in the login
                    // flow.
                    spawn(clone!(@weak obj => async move {
                        obj.send_registration().await;
                    }));
                }
            }),
        );
    }

    fn disable_actions(&self) {
        self.action_set_enabled("login.previous", false);
        self.action_set_enabled("login.next", false);
        self.action_set_enabled("login.use-qr-code", false);
        self.action_set_enabled("login.resend-auth-code", false);
        self.action_set_enabled("login.go-to-forgot-password-page", false);
        self.action_set_enabled("login.recover-password", false);
        self.action_set_enabled("login.show-no-email-access-dialog", false);
        self.action_set_enabled("login.show-delete-account-dialog", false);
        self.action_set_enabled("login.show-tos-dialog", false);
    }

    fn freeze(&self) {
        self.disable_actions();
        self.imp().content.set_sensitive(false);
    }

    fn freeze_with_previous_spinner(&self) {
        self.freeze();

        self.imp().previous_stack.set_visible_child_name("spinner");
    }

    fn freeze_with_next_spinner(&self) {
        self.freeze();

        let imp = self.imp();
        imp.next_stack.set_visible_child(&imp.next_spinner.get());
    }

    fn unfreeze(&self) {
        let imp = self.imp();
        imp.previous_stack.set_visible_child_name("text");
        imp.next_stack.set_visible_child(&imp.next_label.get());
        imp.content.set_sensitive(true);
    }

    async fn send_encryption_key(&self) {
        let imp = self.imp();
        let client_id = imp.client_id.get();
        let result = functions::check_database_encryption_key(String::new(), client_id).await;

        if let Err(err) = result {
            show_error_label(&imp.welcome_page_error_label, &err.message)
        }
    }

    async fn send_phone_number(&self) {
        let imp = self.imp();

        reset_error_label(&imp.welcome_page_error_label);

        let client_id = imp.client_id.get();
        let phone_number = imp.phone_number_input.number();

        // Check if we are already have an account logged in with that phone_number.
        let phone_number_digits = phone_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();

        let session_manager = imp.session_manager.get().unwrap();

        match session_manager.session_index_for(self.use_test_dc(), &phone_number_digits) {
            Some(pos) => {
                // We just figured out that we already have an open session for that account.
                // Therefore we logout the client, with which we wanted to log in and delete its
                // just created database directory.
                log_out(imp.client_id.get()).await;
                imp.session_manager
                    .get()
                    .unwrap()
                    .switch_to_sessions(Some(pos));
            }
            None => {
                let result = functions::set_authentication_phone_number(
                    phone_number.into(),
                    Some(types::PhoneNumberAuthenticationSettings {
                        allow_flash_call: true,
                        allow_missed_call: true,
                        ..Default::default()
                    }),
                    client_id,
                )
                .await;

                self.handle_user_result(
                    result,
                    &imp.welcome_page_error_label,
                    &*imp.phone_number_input,
                );
                imp.phone_number_input.select_number_without_calling_code()
            }
        }
    }

    async fn send_code(&self) {
        let imp = self.imp();

        reset_error_label(&imp.code_error_label);

        let client_id = imp.client_id.get();
        let code = imp.code_entry_row.text().to_string();
        let result = functions::check_authentication_code(code, client_id).await;

        if let Err(err) = result {
            self.handle_user_error(&err, &imp.code_error_label, &*imp.code_entry_row);
        } else {
            // We entered the correct code, so stop resend countdown.
            self.stop_code_next_type_countdown()
        }
    }

    async fn send_registration(&self) {
        let imp = self.imp();

        reset_error_label(&imp.registration_error_label);

        let client_id = imp.client_id.get();
        let first_name = imp.registration_first_name_entry_row.text().to_string();
        let last_name = imp.registration_last_name_entry_row.text().to_string();
        let result = functions::register_user(first_name, last_name, client_id).await;

        self.handle_user_result(
            result,
            &imp.registration_error_label,
            &*imp.registration_first_name_entry_row,
        );
    }

    async fn resend_auth_code(&self) {
        let imp = self.imp();
        let client_id = imp.client_id.get();
        let result = functions::resend_authentication_code(client_id).await;

        if let Err(err) = result {
            if err.code == 8 {
                // Sometimes the user may get a FLOOD_WAIT when he/she wants to resend the
                // authorization code. But then tdlib blocks the resend function for the
                // user, but does not inform us about it by sending an
                // 'AuthorizationState::WaitCode'. Consequently, the user interface would
                // still indicate that we are allowed to resend the code. However, we
                // always get code 8 when we try, indicating that resending does not work.
                // In this case, we automatically disable the resend feature.
                self.update_code_resend_state(None, 0);
            }
            self.handle_user_error(&err, &imp.code_error_label, &*imp.code_entry_row);
        }
    }

    async fn send_password(&self) {
        let imp = self.imp();

        reset_error_label(&imp.password_error_label);

        let client_id = imp.client_id.get();
        let password = imp.password_entry_row.text().to_string();
        let result = functions::check_authentication_password(password, client_id).await;

        self.handle_user_result(result, &imp.password_error_label, &*imp.password_entry_row);
    }

    fn recover_password(&self) {
        let imp = self.imp();

        if imp.password_recovery_expired.get() {
            // We need to tell tdlib to send us the recovery code via mail (again).
            self.freeze();
            imp.password_send_code_stack
                .set_visible_child_name("spinner");

            let client_id = imp.client_id.get();

            spawn(clone!(@weak self as obj => async move {
                let result = functions::request_authentication_password_recovery(client_id).await;
                let imp = obj.imp();

                // Remove the spinner from the button.
                imp.password_send_code_stack.set_visible_child_name("image");

                if result.is_ok() {
                    // Save that we do not need to resend the mail when we enter the recovery
                    // page the next time.
                    imp.password_recovery_expired.set(false);
                    obj.navigate_to_page(
                        "password-recovery-page",
                        [&*imp.password_recovery_code_entry_row],
                        Some(&imp.password_recovery_error_label),
                        Some(&*imp.password_recovery_code_entry_row),
                    );
                } else {
                    obj.update_actions_for_visible_page();
                    // TODO: We also need to handle potiential errors here and inform the user.
                }

                obj.unfreeze();
            }));
        } else {
            // The code has been send already via mail.
            self.navigate_to_page(
                "password-recovery-page",
                [&*imp.password_recovery_code_entry_row],
                Some(&imp.password_recovery_error_label),
                Some(&*imp.password_recovery_code_entry_row),
            );
        }
    }

    fn show_delete_account_dialog(&self) {
        let dialog = adw::MessageDialog::builder()
            .heading(&gettext("Warning"))
            .body(&gettext(
                "You will lose all your chats and messages, along with any media and files you shared!\n\nDo you want to delete your account?",
            ))
            .transient_for(self.root().unwrap().downcast_ref::<gtk::Window>().unwrap())
            .build();

        dialog.add_responses(&[
            ("cancel", &gettext("_Cancel")),
            ("delete", &gettext("_Delete Account")),
        ]);
        dialog.set_default_response(Some("cancel"));
        dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);

        dialog.run_async(
            None,
            clone!(@weak self as obj => move |_, response| {
                if response == "delete" {
                    obj.freeze();
                    let client_id = obj.imp().client_id.get();

                    spawn(clone!(@weak obj => async move {
                        let result = functions::delete_account(
                            String::from("cloud password lost and not recoverable"),
                            client_id,
                        )
                        .await;

                        // Just unfreeze in case of an error, else stay frozen until we are
                        // redirected to the welcome page.
                        if result.is_err() {
                            obj.update_actions_for_visible_page();
                            obj.unfreeze();
                            // TODO: We also need to handle potential errors here and inform the
                            // user.
                        }
                    }));
                } else {
                    obj.imp().password_entry_row.grab_focus();
                }
            }),
        );
    }

    async fn send_password_recovery_code(&self) {
        let imp = self.imp();
        let client_id = imp.client_id.get();
        let recovery_code = imp.password_recovery_code_entry_row.text().to_string();
        let result = functions::recover_authentication_password(
            recovery_code,
            String::new(),
            String::new(),
            client_id,
        )
        .await;

        if let Err(err) = result {
            if err.message == "PASSWORD_RECOVERY_EXPIRED" {
                // The same procedure is used as for the official client (as far as I
                // understood from the code). Alternatively, we could send the user a new
                // code, indicate that and stay on the recovery page.
                imp.password_recovery_expired.set(true);
                self.navigate_to_page::<gtk::Editable, _, _>(
                    "password-page",
                    [],
                    None,
                    Some(&*imp.password_entry_row),
                );
            } else {
                self.handle_user_error(
                    &err,
                    &imp.password_recovery_error_label,
                    &*imp.password_recovery_code_entry_row,
                );
            }
        }
    }

    fn show_no_email_access_dialog(&self) {
        let dialog = adw::MessageDialog::builder()
            .heading(&gettext("Sorry"))
            .body(&gettext(
                "If you can't restore access to the e-mail, your remaining options are either to remember your password or to delete and then recreate your account.",
            ))
            .transient_for(self.root().unwrap().downcast_ref::<gtk::Window>().unwrap())
            .build();

        dialog.add_responses(&[("ok", &gettext("_Ok"))]);
        dialog.set_default_response(Some("ok"));

        dialog.run_async(
            None,
            clone!(@weak self as obj => move |_, _| {
                obj.imp()
                    .password_recovery_code_entry_row
                    .grab_focus();
            }),
        );
    }

    fn use_test_dc(&self) -> bool {
        self.imp()
            .session
            .borrow()
            .as_ref()
            .unwrap()
            .database_info()
            .0
            .use_test_dc
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
}

fn show_error_label(error_label: &gtk::Label, message: &str) {
    error_label.set_text(message);
    error_label.set_visible(true);
}

fn reset_error_label(error_label: &gtk::Label) {
    error_label.set_text("");
    error_label.set_visible(false);
}

fn stringify_auth_code_type(code_type: AuthenticationCodeType) -> String {
    match code_type {
        AuthenticationCodeType::TelegramMessage(_) => gettext("Telegram"),
        AuthenticationCodeType::Sms(_) => gettext("SMS"),
        AuthenticationCodeType::Call(_) => gettext("Call"),
        AuthenticationCodeType::FlashCall(_) => gettext("Flash Call"),
        AuthenticationCodeType::MissedCall(_) => gettext("Missed Call"),
    }
}
