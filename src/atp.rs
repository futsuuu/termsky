use std::{fs::File, path::PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use atrium_api::agent::{store::SessionStore, AtpAgent, Session};
use atrium_xrpc_client::reqwest::ReqwestClientBuilder;
use tokio::sync::mpsc;
use tracing::{event, Level};

use crate::app;

pub enum Request {
    GetSession,
    Login { ident: String, passwd: String },
}

pub enum Response {
    Session(Option<Session>),
    Login { err: Option<String> },
}

pub async fn handler(
    mut rx: mpsc::UnboundedReceiver<Request>,
    tx: mpsc::UnboundedSender<app::Response>,
) -> Result<()> {
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

    while let Some(request) = rx.recv().await {
        let res = match request {
            Request::GetSession => Response::Session(session.clone()),
            Request::Login { ident, passwd } => {
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
                Response::Login { err }
            }
        };

        if tx.send(app::Response::Atp(res)).is_err() {
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
