use grammers_client::{Client, Config, InputMessage, SignInError, Update};
use grammers_client::client::chats::AuthorizationError;
use grammers_client::types::{Dialog, LoginToken, Message};
use grammers_session::FileSession;
use gtk::glib;
use std::sync::Arc;
use tokio::{runtime, task};
use tokio::sync::mpsc;

use crate::config;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub enum EventGTK {
    AccountNotAuthorized,
    AuthorizationError(AuthorizationError),
    NeedConfirmationCode,
    SignInError(SignInError),
    AccountAuthorized,
    ReceivedDialog(Dialog),
    ReceivedMessage(Message),
    NewMessage(Message),
}

pub enum EventTG {
    SendPhoneNumber(String),
    SendConfirmationCode(String),
    RequestDialogs,
    RequestMessages(Arc<Dialog>),
    SendMessage(Arc<Dialog>, InputMessage),
}

pub fn spawn(gtk_sender: glib::Sender<EventGTK>, tg_receiver: mpsc::Receiver<EventTG>) {
    std::thread::spawn(move || {
        let _ = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(start(gtk_sender, tg_receiver));
    });
}

async fn start(gtk_sender: glib::Sender<EventGTK>, mut tg_receiver: mpsc::Receiver<EventTG>) -> Result {
    let api_id = config::TG_API_ID.to_owned();
    let api_hash = config::TG_API_HASH.to_owned();

    let mut client = Client::connect(Config {
        session: FileSession::load_or_create("telegrand.session")?,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    if !client.is_authorized().await? {
        gtk_sender.send(EventGTK::AccountNotAuthorized).unwrap();

        let mut token: Option<LoginToken> = None;
        while let Some(event) = tg_receiver.recv().await {
            match event {
                EventTG::SendPhoneNumber(number) => {
                    match client.request_login_code(&number, api_id, &api_hash).await {
                        Ok(token_) => {
                            token = Some(token_);
                            gtk_sender.send(EventGTK::NeedConfirmationCode).unwrap();
                        }
                        Err(error) => {
                            gtk_sender.send(EventGTK::AuthorizationError(error)).unwrap();
                        }
                    };
                }
                EventTG::SendConfirmationCode(code) => {
                    match client.sign_in(token.as_ref().unwrap(), &code).await {
                        Ok(_) => {
                            // TODO: sign out when closing the app if this fails.
                            client.session().save()?;
                            break;
                        }
                        Err(error) => {
                            gtk_sender.send(EventGTK::SignInError(error)).unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    gtk_sender.send(EventGTK::AccountAuthorized).unwrap();

    let mut client_handle = client.handle();
    let gtk_sender_clone = gtk_sender.clone();
    task::spawn(async move {
        while let Some(updates) = client.next_updates().await.unwrap() {
            for update in updates {
                match update {
                    Update::NewMessage(message) => {
                        gtk_sender_clone.send(EventGTK::NewMessage(message)).unwrap();
                    }
                    _ => {}
                }
            }
        }
    });

    while let Some(event) = tg_receiver.recv().await {
        match event {
            EventTG::RequestDialogs => {
                let mut dialogs = client_handle.iter_dialogs();
                while let Some(dialog) = dialogs.next().await.unwrap() {
                    gtk_sender.send(EventGTK::ReceivedDialog(dialog)).unwrap();
                }
            }
            EventTG::RequestMessages(dialog) => {
                let mut messages = client_handle.iter_messages(dialog.chat()).limit(20);
                while let Some(message) = messages.next().await.unwrap() {
                    gtk_sender.send(EventGTK::ReceivedMessage(message)).unwrap();
                }
            }
            EventTG::SendMessage(dialog, message) => {
                client_handle.send_message(dialog.chat(), message).await?;
            }
            _ => {}
        }
    }

    Ok(())
}
