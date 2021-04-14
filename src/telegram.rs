use grammers_client::{Client, Config, InputMessage, SignInError, Update};
use grammers_client::client::chats::AuthorizationError;
use grammers_client::client::messages::MessageIter;
use grammers_client::types::{Dialog, LoginToken, Message, Photo};
use grammers_client::types::photo_sizes::VecExt;
use grammers_session::FileSession;
use gtk::glib;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::{runtime, task};
use tokio::sync::mpsc;

use crate::config;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub enum TelegramEvent {
    AccountAuthorized,
    AccountNotAuthorized,
    NeedConfirmationCode,
    PhoneNumberError(AuthorizationError),
    ConfirmationCodeError(SignInError),

    RequestedDialog(Dialog, MessageIter),
    RequestedNextMessages(Vec<Message>, i32),
    MessagePhotoDownloaded(PathBuf, i32, i32),

    NewMessage(Message),
}

pub enum GtkEvent {
    SendPhoneNumber(String),
    SendConfirmationCode(String),

    RequestDialogs,
    RequestNextMessages(Arc<Mutex<MessageIter>>, i32),
    DownloadMessagePhoto(Photo, i32, i32),

    SendMessage(Arc<Dialog>, InputMessage),
}

pub fn spawn(tg_sender: glib::Sender<TelegramEvent>, gtk_receiver: mpsc::Receiver<GtkEvent>) {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    std::thread::spawn(move || {
        let result = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(start(tg_sender, gtk_receiver));

        // Panic on error
        // TODO: add automatic reconnection on error
        result.expect("Telegram thread error")
    });
}

pub fn send_gtk_event(gtk_sender: &mpsc::Sender<GtkEvent>, event: GtkEvent) {
    let _ = runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(gtk_sender.send(event));
}

async fn start(tg_sender: glib::Sender<TelegramEvent>, mut gtk_receiver: mpsc::Receiver<GtkEvent>) -> Result {
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
        tg_sender.send(TelegramEvent::AccountNotAuthorized).unwrap();

        let mut token: Option<LoginToken> = None;
        while let Some(event) = gtk_receiver.recv().await {
            match event {
                GtkEvent::SendPhoneNumber(number) => {
                    match client.request_login_code(&number, api_id, &api_hash).await {
                        Ok(token_) => {
                            token = Some(token_);
                            tg_sender.send(TelegramEvent::NeedConfirmationCode).unwrap();
                        }
                        Err(error) => {
                            tg_sender.send(TelegramEvent::PhoneNumberError(error)).unwrap();
                        }
                    };
                }
                GtkEvent::SendConfirmationCode(code) => {
                    match client.sign_in(token.as_ref().unwrap(), &code).await {
                        Ok(_) => {
                            // TODO: sign out when closing the app if this fails
                            client.session().save()?;
                            break;
                        }
                        Err(error) => {
                            tg_sender.send(TelegramEvent::ConfirmationCodeError(error)).unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    tg_sender.send(TelegramEvent::AccountAuthorized).unwrap();

    let mut client_handle = client.handle();
    let tg_sender_clone = tg_sender.clone();
    task::spawn(async move {
        while let Some(updates) = client.next_updates().await.unwrap() {
            for update in updates {
                match update {
                    Update::NewMessage(message) => {
                        tg_sender_clone.send(TelegramEvent::NewMessage(message)).unwrap();
                    }
                    _ => {}
                }
            }
        }
    });

    while let Some(event) = gtk_receiver.recv().await {
        match event {
            GtkEvent::RequestDialogs => {
                let mut dialogs = client_handle.iter_dialogs();
                while let Some(dialog) = dialogs.next().await.unwrap() {
                    let iterator = client_handle.iter_messages(dialog.chat());
                    tg_sender.send(TelegramEvent::RequestedDialog(dialog, iterator)).unwrap();
                }
            }
            GtkEvent::RequestNextMessages(iterator, chat_id) => {
                // Return the next 20 messages
                let mut iterator = iterator.lock().unwrap();
                let mut messages = Vec::<Message>::new();
                for _ in 0..20 {
                    if let Some(message) = iterator.next().await.unwrap() {
                        // If there´s a photo, download the lowest resolution
                        // version of the photo to use it for the preview while
                        // the high resolution one is downloading.
                        if let Some(photo) = message.photo() {
                            // Create base directory structure for the photo
                            let path = glib::get_user_special_dir(glib::UserDirectory::Downloads);
                            let path = path.join(format!("Telegrand/{}", chat_id));
                            glib::mkdir_with_parents(&path, 0o744);

                            // Download low resolution photo in the directory
                            let path = path.join(format!("{}.jpg", photo.id()));
                            photo.thumbs().iter().min_by_key(|x| x.size())
                                .unwrap().download(&path).await;
                        }

                        messages.push(message);
                    }
                }
                tg_sender.send(TelegramEvent::RequestedNextMessages(messages, chat_id)).unwrap();
            }
            GtkEvent::DownloadMessagePhoto(photo, chat_id, message_id) => {
                // Create base directory structure for the photo
                let path = glib::get_user_special_dir(glib::UserDirectory::Downloads);
                let path = path.join(format!("Telegrand/{}", chat_id));
                glib::mkdir_with_parents(&path, 0o744);

                // Download high resolution photo in the directory
                let path = path.join(format!("{}.jpg", photo.id()));
                photo.thumbs().largest().unwrap().download(&path).await;

                // Tell gtk that the photo has been downloaded
                tg_sender.send(TelegramEvent::MessagePhotoDownloaded(
                    path, chat_id, message_id)).unwrap();
            }
            GtkEvent::SendMessage(dialog, message) => {
                client_handle.send_message(dialog.chat(), message).await?;
            }
            _ => {}
        }
    }

    Ok(())
}
