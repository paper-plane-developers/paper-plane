//! Widget for managing sessions.
//!
//! This widget can be considered as the main view. It is directly subordinate to the application
//! window and takes care of managing sessions. In the following sections it is described what this
//! includes.
//!
//! # Adding new sessions
//! The `SessionManager` directs it to [`Login`](struct@crate::login::Login) as soon as the user
//! wants to add a new session within the function `add_new_session`. Login returns control to the
//! session manager if either the new session is ready, logging into the new session is aborted by
//! the user, or it is noticed that the phone number is already logged in. In order ro prevent
//! logging in twice in the same account by an qr code, the function
//! [`SessionManager::logged_in_users()`] is provided that is used by `login.rs` to extract user
//! ids and pass them to tdlib.
//!
//! # Adding existing sessions
//! The `SessionManager` analyzes the individual database directories in the Telegrand data
//! directory to see which sessions can be logged in directly using
//! [`SessionManager::add_existing_session()`]. To do this, it checks the presence of a `td.binlog`
//! or a `td_test.binlog` file.
//!
//! # Destroying sessions
//! This is realized by first logging out the client and then deleting the database directory once
//! the `AuthorizationState::Closed` event has been received for that session.
//! Destroying sessions happens in different places: When the login is canceled, When the QR code
//! is canceled, when a logged in session is logged out, and when the session is removed from
//! another device.
//!
//! # Remembering recently used sessions
//! In order to remember the order in which the user selected the sessions, the `SessionManager`
//! uses a gsettings key value pair.

use futures::TryFutureExt;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use std::borrow::Borrow;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tdlib::enums::{self, AuthorizationState, Update};
use tdlib::functions;
use tdlib::types::{self, UpdateAuthorizationState};

use crate::session::{Session, User};
use crate::utils::{block_on, data_dir, log_out, send_tdlib_parameters, spawn};
use crate::APPLICATION_OPTS;

/// Struct for representing a TDLib client.
#[derive(Clone, Debug)]
pub(crate) struct Client {
    /// The `Session` of this client.
    pub(crate) session: Session,
    /// The `ClientState` of this client.
    pub(crate) state: ClientState,
}

impl Client {
    fn database_dir_base_name(&self) -> &str {
        &self.session.database_info().0.directory_base_name
    }
}

/// Enum for storing information about the state of an TDLib client.
#[derive(Clone, Debug)]
pub(crate) enum ClientState {
    /// The client is currently in the authorization state. Every client, even those that were
    /// logged in during a previous run of the application will need to go through this state
    /// again.
    Auth {
        /// Whether there is a chance that the client is already authorized.
        /// This will be set to `true` for all the sessions found in the data directory at
        /// application start because we assume that they will get the `AuthorizationState::Ready`
        /// without user interaction as they were probably logged in before.
        /// So, we use this information to bypass the login process (phone number -> auth code ->
        /// ... -> ready) and to just wait for the `AuthorizationState::Ready` update.
        maybe_authorized: bool,
    },
    /// The client is logged and has a `Session`.
    LoggedIn,
    /// The client is currently in the process of logging out
    LoggingOut,
}

mod imp {
    use super::*;

    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;

    use crate::Login;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/melix99/telegrand/ui/session-manager.ui")]
    pub(crate) struct SessionManager {
        /// The order of the recently used sessions. The string stored in the `Vec` represents the
        /// session's database directory name.
        pub(super) recently_used_sessions: RefCell<Vec<String>>,
        /// The number sessions to load/handle at application start. This number will indirectly be
        /// determined in [`analyze_data_dir()`]
        pub(super) initial_sessions_to_handle: Cell<u32>,
        pub(super) clients: RefCell<HashMap<i32, Client>>,
        pub(super) wait_me_handlers: RefCell<HashMap<i32, glib::SignalHandlerId>>,
        #[template_child]
        pub(super) main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) login: TemplateChild<Login>,
        #[template_child]
        pub(super) sessions: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SessionManager {
        const NAME: &'static str = "SessionManager";
        type Type = super::SessionManager;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SessionManager {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.login.set_session_manager(obj.clone());

            // Take action when the active client changed.
            self.main_stack
                .connect_visible_child_notify(clone!(@weak obj => move |main_stack| {
                    if main_stack.visible_child().unwrap().downcast_ref::<gtk::Stack>().is_some() {
                        obj.on_active_session_changed();
                    }
                }));

            // Take action when the active client changed.
            self.sessions.connect_visible_child_notify(
                clone!(@weak obj => move |_| obj.on_active_session_changed()),
            );

            // ####################################################################################
            // # Load the sessions from the data directory.                                       #
            // ####################################################################################
            match analyze_data_dir() {
                // TODO: Should we show a dialog in this case instead of just bailing out
                // silently?
                Err(e) => panic!("Could not initialize data directory: {}", e),
                Ok(datadir_state) => match datadir_state {
                    DatadirState::Empty => {
                        obj.add_new_session(APPLICATION_OPTS.get().unwrap().test_dc);
                    }
                    DatadirState::HasSessions {
                        recently_used_sessions,
                        database_infos,
                    } => {
                        let imp = obj.imp();

                        imp.recently_used_sessions.replace(recently_used_sessions);

                        imp.initial_sessions_to_handle
                            .set(database_infos.len() as u32);

                        database_infos.into_iter().for_each(|database_info| {
                            obj.add_existing_session(database_info);
                        });
                    }
                },
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.main_stack.unparent();
        }
    }

    impl WidgetImpl for SessionManager {}
}

glib::wrapper! {
    pub(crate) struct SessionManager(ObjectSubclass<imp::SessionManager>)
        @extends gtk::Widget;
}

impl SessionManager {
    /// Returns the active client id if it is logged in or `None` if it isn't logged in
    /// (e.g. during authorization).
    fn active_logged_in_client_id(&self) -> Option<i32> {
        let imp = self.imp();

        imp.main_stack
            .visible_child()
            .filter(|widget| widget == imp.sessions.get().upcast_ref::<gtk::Widget>())
            .map(|_| {
                imp.sessions
                    .visible_child()
                    .unwrap()
                    .downcast_ref::<Session>()
                    .unwrap()
                    .client_id()
            })
    }

    /// Function that returns all currently logged in users.
    pub(crate) fn logged_in_users(&self) -> Vec<User> {
        let sessions = self.sessions();

        (0..sessions.n_items())
            .map(|pos| {
                sessions
                    .item(pos)
                    .unwrap()
                    .downcast::<gtk::StackPage>()
                    .unwrap()
                    .child()
                    .downcast::<Session>()
                    .unwrap()
                    .me()
            })
            .collect()
    }

    /// Returns the `Client` for the given client id.
    pub(crate) fn client(&self, client_id: i32) -> Option<Client> {
        self.imp().clients.borrow().get(&client_id).cloned()
    }

    /// Returns the index of the logged in session that matches the passed parameters or `None` if
    /// no matching session was found.
    ///
    /// This function is mainly used to check if a logged in session already exists for an account
    /// to prevent two sessions for the same account.
    pub(crate) fn session_index_for(
        &self,
        on_test_dc: bool,
        phone_number_digits: &str,
    ) -> Option<u32> {
        let sessions = self.sessions();

        (0..sessions.n_items()).find_map(|pos| {
            let session = sessions
                .item(pos)
                .unwrap()
                .downcast::<gtk::StackPage>()
                .unwrap()
                .child()
                .downcast::<Session>()
                .unwrap();

            if on_test_dc == session.database_info().0.use_test_dc
                && session.me().phone_number().replace(' ', "") == phone_number_digits
            {
                Some(pos)
            } else {
                None
            }
        })
    }

    /// Function for switching the main stack to the active sessions. It also will switch to the
    /// given position if it has some value.
    ///
    /// This function has basically two callers both in `Login`:
    /// First, it will be called when the `back` button is pressed on the phone number page to go
    /// back to the last session. Secondly, it will be called with a position if the phone number
    /// entered already has a session.
    pub(crate) fn switch_to_sessions(&self, pos: Option<u32>) {
        let imp = self.imp();
        imp.main_stack.set_visible_child(&*imp.sessions);
        if let Some(pos) = pos {
            imp.sessions.pages().select_item(pos, true);
        }
    }

    /// Returns sessions as selection model.
    ///
    /// Is mainly used by `Login` to check whether the back button should be visible on the phone
    /// number page and to check the session' phone numbers in order to not have 2 sessions of the
    /// same account.
    pub(crate) fn sessions(&self) -> gtk::SelectionModel {
        self.imp().sessions.pages()
    }

    /// This functions will be invoked when the active client has changed.
    /// It does:
    ///   1. Update the online status of the clients
    ///   2. Update the order of the recently used sessions
    ///
    /// This is invoked when the visible child of the main stack or the sessions stack changes.
    fn on_active_session_changed(&self) {
        let imp = self.imp();

        if let Some(session) = imp
            .sessions
            .visible_child()
            .and_then(|widget| widget.downcast::<Session>().ok())
        {
            self.transfer_online_status(session.client_id());

            if imp.main_stack.visible_child() == Some(imp.sessions.clone().upcast()) {
                let database_dir_base_name = session.database_info().0.directory_base_name.clone();

                {
                    let mut recently_used_sessions = imp.recently_used_sessions.borrow_mut();
                    remove_from_vec(&mut *recently_used_sessions, &database_dir_base_name);
                    recently_used_sessions.push(database_dir_base_name);
                }

                self.save_recently_used_sessions();
            }
        }
    }

    /// Sets the online status for the active logged in client. This will be called from the
    /// application `Window` when its active state has changed.
    pub(crate) async fn set_active_client_online(&self, value: bool) {
        if let Some(client_id) = self.active_logged_in_client_id() {
            let result = set_online(client_id, value).await;
            if let Err(e) = result {
                log::warn!("Error setting a client online state: {:?}", e);
            }
        }
    }

    /// Transfers the online status to this given new active client id. All other clients' online
    /// status are set to `false`.
    fn transfer_online_status(&self, active_client_id: i32) {
        self.imp()
            .clients
            .borrow()
            .values()
            .filter_map(|client| match client.state {
                ClientState::LoggedIn => Some(client.session.client_id()),
                _ => None,
            })
            .for_each(|client_id| {
                spawn(async move {
                    let result = set_online(
                        client_id,
                        // Session switching is only possible when the window is active.
                        client_id == active_client_id,
                    )
                    .await;

                    if let Err(e) = result {
                        log::warn!("Error setting a client online state: {:?}", e);
                    }
                });
            });
    }

    /// This function is used to add/load an existing session that already had the
    /// `AuthorizationState::Ready` state from a previous application run.
    pub(crate) fn add_existing_session(&self, database_info: DatabaseInfo) {
        let client_id = tdlib::create_client();

        self.imp().clients.borrow_mut().insert(
            client_id,
            Client {
                session: Session::new(client_id, database_info),
                state: ClientState::Auth {
                    // Important: Here, we basically say that we just want to wait for
                    // `AuthorizationState::Ready` and skip the login process.
                    maybe_authorized: true,
                },
            },
        );

        spawn(async move {
            send_log_level(client_id).await;

            // TODO: Hopefully we'll support animated emoji at some point
            let result = disable_animated_emoji(client_id, true).await;
            if let Err(e) = result {
                log::warn!("Error disabling animated emoji: {:?}", e);
            }
        });
    }

    /// This function is used to add a new session for a so far unknown account. This means it will
    /// go through the login process.
    pub(crate) fn add_new_session(&self, use_test_dc: bool) {
        let client_id = tdlib::create_client();
        self.init_new_session(client_id, use_test_dc);
        spawn(async move {
            send_log_level(client_id).await;

            // TODO: Hopefully we'll support animated emoji at some point
            let result = disable_animated_emoji(client_id, true).await;
            if let Err(e) = result {
                log::warn!("Error disabling animated emoji: {:?}", e);
            }
        });
    }

    /// This function initializes everything that's needed for adding a new session for the given
    /// client id.
    fn init_new_session(&self, client_id: i32, use_test_dc: bool) {
        let imp = self.imp();

        let database_info = DatabaseInfo {
            directory_base_name: generate_database_dir_base_name(),
            use_test_dc,
        };

        let session = Session::new(client_id, database_info);

        imp.clients.borrow_mut().insert(
            client_id,
            Client {
                session: session.clone(),
                state: ClientState::Auth {
                    // Important: Here, we state that this client will have to go through the login
                    // process.
                    maybe_authorized: false,
                },
            },
        );

        imp.login.login_client(client_id, session);

        imp.main_stack.set_visible_child(&*imp.login);
    }

    /// This function is called when a session is in the process of logging out.
    ///
    /// Among other things, it switches to the last session or to the login page, if called from a
    /// logged in session. Furthermore, the recently used sessions order file will be overwritten.
    fn set_session_logging_out(&self, client: &Client) {
        if let ClientState::LoggedIn = client.state {
            let imp = self.imp();

            imp.sessions.remove(&client.session);

            let database_dir_base_name = client.database_dir_base_name();

            if !remove_from_vec(
                &mut *imp.recently_used_sessions.borrow_mut(),
                database_dir_base_name,
            ) {
                log::warn!(
                    "Could not remove session directory base name from recently used sessions: {}",
                    database_dir_base_name
                );
            }

            if imp.sessions.pages().n_items() > 0 {
                // Important: This must not put as an expression in the `select_item` method
                // but rather stay here as a statement. Else, a borrow error will occur.
                let active_session_index = imp
                    .recently_used_sessions
                    .borrow()
                    .last()
                    .and_then(|database_dir_base_name| {
                        (0..self.sessions().n_items()).find_map(|pos| {
                            let session = self
                                .sessions()
                                .item(pos)
                                .unwrap()
                                .downcast::<gtk::StackPage>()
                                .unwrap()
                                .child()
                                .downcast::<Session>()
                                .unwrap();

                            if &session.database_info().0.directory_base_name
                                == database_dir_base_name
                            {
                                Some(pos)
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or_default();

                imp.sessions.pages().select_item(active_session_index, true);
            } else {
                // There are no sessions left. Thus go back to login.
                self.add_new_session(APPLICATION_OPTS.get().unwrap().test_dc);
            }

            // save recently used sessions
            self.save_recently_used_sessions();
        }
    }

    /// Function cleaning up, which is called by the application windows on closing. It sets all
    /// clients offline.
    pub(crate) fn close_clients(&self) {
        // Create a future to close the sessions.
        let close_sessions_future = futures::future::join_all(
            self.imp()
                .clients
                .borrow()
                .iter()
                .filter_map(|(client_id, client)| match client.state {
                    ClientState::Auth { .. } | ClientState::LoggedIn => Some(client_id),
                    _ => None,
                })
                .cloned()
                .map(|client_id| {
                    set_online(client_id, false).and_then(move |_| functions::close(client_id))
                }),
        );

        // Block on that future, else the window closes before they are finished!!!
        block_on(async {
            close_sessions_future.await.into_iter().for_each(|result| {
                if let Err(e) = result {
                    log::warn!("Error on closing client: {:?}", e);
                }
            });
        });
    }

    pub(crate) fn handle_update(&self, update: Update, client_id: i32) {
        match update {
            Update::AuthorizationState(update) => {
                self.handle_authorization_state(update, client_id);
            }
            update => {
                let clients = self.imp().clients.borrow();
                let session = clients.get(&client_id).unwrap().session.clone();

                // Drop the reference before giving hand over control.
                drop(clients);
                session.handle_update(update);
            }
        }
    }

    /// This function is used to log in an account. Within the function it is first checked if the
    /// client needs to authorize. Then the `AuthorizationState` is delegated to the `Login`
    /// widget. Otherwise, if the client was already authorized from previous a application run,
    /// the session is created directly in this function.
    fn handle_authorization_state(&self, update: UpdateAuthorizationState, client_id: i32) {
        let imp = self.imp();

        if let AuthorizationState::Closed = update.authorization_state {
            let client = imp.clients.borrow_mut().remove(&client_id).unwrap();
            if let ClientState::LoggingOut = client.state {
                let database_dir_base_name = client.database_dir_base_name().to_owned();
                if let Err(e) = fs::remove_dir_all(data_dir().join(database_dir_base_name)) {
                    log::error!("Error on on removing database directory: {}", e);
                }
            }
            return;
        }

        let client = self.client(client_id).unwrap();

        if let AuthorizationState::LoggingOut = update.authorization_state {
            self.set_session_logging_out(&client);
            imp.clients.borrow_mut().insert(
                client_id,
                Client {
                    session: client.session,
                    state: ClientState::LoggingOut,
                },
            );

            return;
        }

        if let ClientState::Auth { maybe_authorized } = client.state {
            if !maybe_authorized {
                imp.login
                    .set_authorization_state(update.authorization_state);
            } else {
                // Client doesn't need to authorize. So we can skip the login procedure.
                match &update.authorization_state {
                    AuthorizationState::WaitTdlibParameters => {
                        let database_info = client.session.database_info().0.clone();
                        spawn(async move {
                            let result = send_tdlib_parameters(client_id, &database_info).await;
                            if let Err(e) = result {
                                panic!("Error on sending tdlib parameters: {:?}", e);
                            }
                        });
                    }
                    AuthorizationState::WaitEncryptionKey(_) => {
                        spawn(async move {
                            let result =
                                functions::check_database_encryption_key(String::new(), client_id)
                                    .await;
                            if let Err(e) = result {
                                panic!("Error on sending encryption key: {:?}", e);
                            }
                        });
                    }
                    AuthorizationState::Ready => {
                        let is_last_used = imp
                            .recently_used_sessions
                            .borrow()
                            .iter()
                            .rev()
                            .next()
                            .map(|last| client.database_dir_base_name() == last)
                            .unwrap_or_default();

                        spawn(clone!(@weak self as obj => async move {
                            obj.add_logged_in_session(client_id, client.session, is_last_used)
                                .await;
                        }));
                    }
                    _ => {
                        let database_info = client.session.database_info().0.clone();

                        // Our assumption that the database's session we found at application start
                        // would not need to authorize was wrong. So we handle it correctly.
                        if APPLICATION_OPTS.get().unwrap().test_dc == database_info.use_test_dc
                            && imp.initial_sessions_to_handle.get() == 1
                        {
                            // Handle it over to `login.rs`.

                            // Overwrite Client.
                            imp.clients.borrow_mut().insert(
                                client_id,
                                Client {
                                    session: client.session.clone(),
                                    state: ClientState::Auth {
                                        maybe_authorized: false,
                                    },
                                },
                            );

                            imp.login.login_client(client_id, client.session);

                            imp.login
                                .set_authorization_state(update.authorization_state);
                        } else {
                            spawn(async move {
                                log_out(client_id).await;
                            });
                        }
                    }
                }
            }
        }
    }

    /// Function that is used to overwrite the recently used sessions file.
    fn save_recently_used_sessions(&self) {
        let settings = gio::Settings::new(crate::config::APP_ID);
        if let Err(e) = settings.set_strv(
            "recently-used-sessions",
            self.imp()
                .recently_used_sessions
                .borrow()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice(),
        ) {
            log::warn!(
                "Failed to save value for gsettings key 'recently-used-sessions': {}",
                e
            );
        }
    }

    /// Within this function a new `Session` is created based on the passed client id. This session
    /// is then added to the session stack.
    pub(crate) async fn add_logged_in_session(
        &self,
        client_id: i32,
        session: Session,
        visible: bool,
    ) {
        let enums::User::User(me) = functions::get_me(client_id).await.unwrap();

        match session.user_list().try_get(me.id) {
            None => {
                log::warn!("Own user has not yet arrived. Waiting…");
                let wait_me_handler = session.user_list().connect_items_changed(
                    clone!(@weak self as obj, @weak session => move |list, _, _, added| {
                        if added > 0 {
                            if let Some(me) = list.try_get(me.id) {
                                log::warn!("Own user arrived.");
                                obj.add_logged_in_session_(client_id, &session, &me, visible);
                            }
                        }
                    }),
                );
                self.imp()
                    .wait_me_handlers
                    .borrow_mut()
                    .insert(client_id, wait_me_handler);
            }
            Some(me) => {
                self.add_logged_in_session_(client_id, &session, &me, visible);
            }
        }
    }

    async fn enable_notifications(&self, client_id: i32) {
        let result = functions::set_option(
            "notification_group_count_max".to_string(),
            Some(enums::OptionValue::Integer(types::OptionValueInteger {
                value: 5,
            })),
            client_id,
        )
        .await;

        if let Err(e) = result {
            log::warn!(
                "Error setting the notification_group_count_max option: {:?}",
                e
            );
        }
    }

    fn add_logged_in_session_(&self, client_id: i32, session: &Session, me: &User, visible: bool) {
        let imp = self.imp();

        if let Some(handler) = imp.wait_me_handlers.borrow_mut().remove(&client_id) {
            session.user_list().disconnect(handler);
        }

        session.set_me(me);
        session.fetch_chats();

        imp.sessions.add_child(session);
        session.set_sessions(&imp.sessions.pages());

        imp.clients.borrow_mut().insert(
            client_id,
            Client {
                session: session.clone(),
                state: ClientState::LoggedIn,
            },
        );

        let auth_session_present = imp
            .clients
            .borrow()
            .values()
            .any(|client| matches!(client.state, ClientState::Auth { .. }));

        if (imp.main_stack.visible_child() != Some(imp.sessions.clone().upcast())
            && !auth_session_present)
            || visible
        {
            imp.sessions.set_visible_child(session);
            imp.main_stack.set_visible_child(&*imp.sessions);
        }

        // Enable notifications for this client
        spawn(clone!(@weak self as obj => async move {
            obj.enable_notifications(client_id).await;
        }));
    }

    pub(crate) fn select_chat(&self, client_id: i32, chat_id: i64) {
        if let Some(client) = self.imp().clients.borrow().get(&client_id) {
            if let ClientState::LoggedIn = client.state {
                client.session.select_chat(chat_id);
            }
        }
    }

    pub(crate) fn handle_paste_action(&self) {
        if let Some(client_id) = self.active_logged_in_client_id() {
            let clients = self.imp().clients.borrow();
            let client = clients.get(&client_id).unwrap();
            if let ClientState::LoggedIn = client.state {
                client.session.handle_paste_action();
            }
        }
    }

    pub(crate) fn begin_chats_search(&self) {
        if let Some(client_id) = self.active_logged_in_client_id() {
            let clients = self.imp().clients.borrow();
            let client = clients.get(&client_id).unwrap();
            if let ClientState::LoggedIn = client.state {
                client.session.begin_chats_search();
            }
        }
    }
}

/// A struct for storing information about a session's database.
#[derive(Clone, Debug)]
pub(crate) struct DatabaseInfo {
    // The base name of the database directory.
    pub(crate) directory_base_name: String,
    // Whether this database uses a test dc.
    pub(crate) use_test_dc: bool,
}

/// A struct for representing the state of the data directory.
#[derive(Debug)]
pub(crate) enum DatadirState {
    /// There are no sessions at all. This probably means that the application is started for the
    /// first time or the data directory has been deleted by the user.
    Empty,
    /// There were several sessions found in the data directory.
    HasSessions {
        /// The `DatabaseInfos`
        database_infos: Vec<DatabaseInfo>,
        /// The order of the recently used sessions that will be read from a gsettings value.
        recently_used_sessions: Vec<String>,
    },
}

/// This function analyzes the data directory.
///
/// First, it checks whether the directory exists. It will create it and return immediately if
/// it doesn't.
///
/// If the data directory exists, information about the sessions is gathered. This is reading the
/// recently used sessions file and checking the individual session's database directory.
fn analyze_data_dir() -> Result<DatadirState, anyhow::Error> {
    if !data_dir().exists() {
        // Create the Telegrand data directory if it does not exist and return.
        fs::create_dir_all(&data_dir())?;
        return Ok(DatadirState::Empty);
    }

    // All directories with the result of reading the session info file.
    let database_infos = fs::read_dir(&data_dir())?
        // Remove entries with error
        .filter_map(|res| res.ok())
        // Only consider directories.
        .filter(|entry| entry.path().is_dir())
        // Only consider directories with a "*.binlog" file
        .filter_map(|entry| {
            if entry.path().join("td.binlog").is_file() {
                return Some((entry, false));
            } else if entry.path().join("td_test.binlog").is_file() {
                return Some((entry, true));
            }
            None
        })
        .map(|(entry, use_test_dc)| DatabaseInfo {
            directory_base_name: entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            use_test_dc,
        })
        .collect::<Vec<_>>();

    if database_infos.is_empty() {
        Ok(DatadirState::Empty)
    } else {
        let mut recently_used_sessions = gio::Settings::new(crate::config::APP_ID)
            .strv("recently-used-sessions")
            .into_iter()
            .map(glib::GString::into)
            .collect::<Vec<_>>();

        // Remove invalid database directory base names from recently used sessions.
        recently_used_sessions.retain(|database_dir_base_name| {
            database_infos
                .iter()
                .any(|database_info| &database_info.directory_base_name == database_dir_base_name)
        });

        Ok(DatadirState::HasSessions {
            recently_used_sessions,
            database_infos,
        })
    }
}

/// This function generates a new database directory name based on the current UNIX system time
/// (e.g. db1638487692420). In the very unlikely case that a name is already taken it tries to
/// append a number at the end.
fn generate_database_dir_base_name() -> String {
    let database_dir_base_name = format!(
        "db{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    );

    // Just to be sure!
    if data_dir().join(&database_dir_base_name).exists() {
        (2..)
            .map(|count| format!("{}_{}", database_dir_base_name, count))
            .find(|alternative_base_name| !data_dir().join(alternative_base_name).exists())
            .unwrap()
    } else {
        database_dir_base_name
    }
}

/// Helper function for setting the online status of a client.
async fn set_online(client_id: i32, value: bool) -> Result<enums::Ok, types::Error> {
    functions::set_option(
        "online".to_string(),
        Some(enums::OptionValue::Boolean(types::OptionValueBoolean {
            value,
        })),
        client_id,
    )
    .await
}

/// Helper function to enable/disable animated emoji for a client.
async fn disable_animated_emoji(client_id: i32, value: bool) -> Result<enums::Ok, types::Error> {
    functions::set_option(
        "disable_animated_emoji".to_string(),
        Some(enums::OptionValue::Boolean(types::OptionValueBoolean {
            value,
        })),
        client_id,
    )
    .await
}

/// Helper function for setting the tdlib log level.
async fn send_log_level(client_id: i32) {
    let result = functions::set_log_verbosity_level(
        if log::log_enabled!(log::Level::Trace) {
            5
        } else if log::log_enabled!(log::Level::Debug) {
            4
        } else if log::log_enabled!(log::Level::Info) {
            3
        } else if log::log_enabled!(log::Level::Warn) {
            2
        } else {
            0
        },
        client_id,
    )
    .await;
    if let Err(e) = result {
        log::warn!("Error setting the tdlib log level: {:?}", e);
    }
}

/// Helper function for removing an element from a [`Vec`] based on an equality comparison.
fn remove_from_vec<T, Q: ?Sized>(vec: &mut Vec<T>, to_remove: &Q) -> bool
where
    T: Borrow<Q>,
    Q: Eq,
{
    match vec.iter().position(|elem| elem.borrow() == to_remove) {
        Some(pos) => {
            vec.remove(pos);
            true
        }
        None => false,
    }
}
