use std::{fs::File, path::PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use atrium_api::agent::{store::SessionStore, AtpAgent, Session};
use atrium_xrpc_client::reqwest::ReqwestClientBuilder;

use crate::{
    command::{Command, CommandRx},
    event::{Event, EventTx},
};

pub fn start(command_rx: CommandRx, event_tx: EventTx) {
    tokio::spawn(command_handler(command_rx, event_tx));
}

async fn command_handler(mut command_rx: CommandRx, event_tx: EventTx) -> Result<()> {
    const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let xrpc_client = ReqwestClientBuilder::new("https://bsky.social")
        .client(client)
        .build();
    let session_store = FileStore::new()?;
    let agent = AtpAgent::new(xrpc_client, session_store.clone());

    let session = if let Some(session) = session_store.get_session().await {
        agent
            .resume_session(session.clone())
            .await
            .ok()
            .and(Some(session))
    } else {
        None
    };

    loop {
        if command_rx.changed().await.is_err() {
            break;
        }
        let event = match *command_rx.borrow_and_update() {
            Command::GetSession => Event::GetSession(session.clone()),
            _ => continue,
        };
        if event_tx.send(event).is_err() {
            break;
        }
    }

    Ok(())
}

#[derive(Clone)]
struct FileStore(PathBuf);

impl FileStore {
    pub fn new() -> Result<Self> {
        Ok(Self(crate::utils::local_data_dir()?.join("session.json")))
    }

    fn open_file(&self) -> Option<File> {
        File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.0)
            .ok()
    }
}

#[async_trait]
impl SessionStore for FileStore {
    async fn get_session(&self) -> Option<Session> {
        self.open_file()
            .and_then(|f| serde_json::from_reader(f).ok())
    }

    async fn set_session(&self, session: Session) {
        self.clear_session().await;
        self.open_file()
            .and_then(|f| serde_json::to_writer(f, &session).ok());
    }

    async fn clear_session(&self) {
        tokio::fs::remove_file(&self.0).await.ok();
    }
}
