use std::{fs::File, path::PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use atrium_api::agent::{store::SessionStore, AtpAgent, Session};
use atrium_xrpc_client::reqwest::ReqwestClientBuilder;
use tracing::{event, Level};

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

    let mut session = if let Some(session) = session_store.get_session().await {
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
            event!(Level::WARN, "command channel is closed");
            break;
        }
        let command = command_rx.borrow_and_update().clone();
        let event = match command {
            Command::GetSession => Event::Session(session.clone()),
            Command::Login { ident, passwd } => {
                let result = agent.login(ident, passwd).await;
                let err = match result {
                    Ok(s) => {
                        event!(Level::INFO, "login successfully");
                        session = Some(s);
                        None
                    }
                    Err(err) => {
                        event!(Level::WARN, "login failed: {err}");
                        Some(String::from("login failed"))
                    }
                };
                Event::Login { err }
            }
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
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.0);
        if file.is_err() {
            event!(Level::ERROR, "failed to open the session file");
        }
        file.ok()
    }
}

#[async_trait]
impl SessionStore for FileStore {
    async fn get_session(&self) -> Option<Session> {
        self.open_file()
            .and_then(|f| serde_json::from_reader(f).ok())
    }

    async fn set_session(&self, session: Session) {
        self.open_file()
            .and_then(|f| serde_json::to_writer(f, &session).ok());
    }

    async fn clear_session(&self) {
        tokio::fs::remove_file(&self.0).await.ok();
    }
}
