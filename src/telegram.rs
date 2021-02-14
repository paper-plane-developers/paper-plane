use grammers_client::{Client, Config};
use grammers_session::FileSession;
use gtk::glib;
use tokio::runtime;

use crate::config;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub enum MessageGTK {
    ShowAddAccountWindow,
}

pub fn spawn(sender: glib::Sender<MessageGTK>) {
    std::thread::spawn(move || {
        let _ = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(start(sender));
    });
}

async fn start(sender: glib::Sender<MessageGTK>) -> Result<()> {
    let api_id = config::TG_API_ID.to_owned();
    let api_hash = config::TG_API_HASH.to_owned();

    println!("Connecting to Telegram...");
    let mut client = Client::connect(Config {
        session: FileSession::load_or_create("telegrand.session")?,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await?;
    println!("Connected!");

    if !client.is_authorized().await? {
        let _ = sender.send(MessageGTK::ShowAddAccountWindow);
    }

    Ok(())
}
