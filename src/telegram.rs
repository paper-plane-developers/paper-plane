use grammers_client::{Client, Config, Update};
use grammers_client::types::{Dialog, Message};
use grammers_session::FileSession;
use gtk::glib;
use std::sync::mpsc;
use tokio::{runtime, task};

use crate::config;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub enum EventGTK {
    AccountNotAuthorized,
    NeedConfirmationCode,
    SuccessfullySignedIn,
    LoadDialog(Dialog),
    NewMessage(Message),
}

pub enum EventTG {
    SendPhoneNumber(String),
    SendConfirmationCode(String),
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

async fn start(gtk_sender: glib::Sender<EventGTK>, tg_receiver: mpsc::Receiver<EventTG>) -> Result {
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

        let mut event = tg_receiver.recv().unwrap();
        let mut signed_in = false;

        while !signed_in {
            if let EventTG::SendPhoneNumber(ref number) = event {
                match client.request_login_code(&number, api_id, &api_hash).await {
                    Ok(token) => {
                        gtk_sender.send(EventGTK::NeedConfirmationCode).unwrap();
                        event = tg_receiver.recv().unwrap();

                        if let EventTG::SendConfirmationCode(ref code) = event {
                            match client.sign_in(&token, &code).await {
                                Ok(_) => {
                                    gtk_sender.send(EventGTK::SuccessfullySignedIn).unwrap();
                                    signed_in = true;
                                }
                                Err(e) => panic!(e)
                            }
                        }
                    }
                    Err(e) => panic!(e)
                };
            } else {
                event = tg_receiver.recv().unwrap();
            }
        }

        // TODO: sign out when closing the app if this fails.
        client.session().save()?;
    }

    let gtk_sender_clone = gtk_sender.clone();
    let client_handle = client.handle();
    task::spawn(async move {
        let mut dialogs = client_handle.iter_dialogs();
        while let Some(dialog) = dialogs.next().await.unwrap() {
            gtk_sender_clone.send(EventGTK::LoadDialog(dialog)).unwrap();
        }
    });

    while let Some(updates) = client.next_updates().await? {
        for update in updates {
            match update {
                Update::NewMessage(message) => {
                    gtk_sender.send(EventGTK::NewMessage(message)).unwrap();
                }
                _ => {}
            }
        }
    }

    Ok(())
}
