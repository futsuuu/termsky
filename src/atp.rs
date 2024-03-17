use std::{fmt, fs::File, path::PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use atrium_api::{
    agent::{store::SessionStore, AtpAgent, Session},
    app::bsky,
};
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use tokio::sync::mpsc;
use tracing::instrument;

use crate::app;

#[derive(Clone)]
pub enum Request {
    GetSession,
    GetTimeline(bsky::feed::get_timeline::Parameters),
    Login { ident: String, passwd: String },
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Request::GetSession => f.write_str("GetSession"),
            Request::GetTimeline(params) => f.debug_tuple("GetTimeline").field(&params).finish(),
            Request::Login {
                ident,
                passwd: _passwd,
            } => f.debug_struct("Login").field("ident", &ident).finish(),
        }
    }
}

pub enum Response {
    Session(Option<Session>),
    Timeline(Result<bsky::feed::get_timeline::Output>),
    Login(Result<()>),
}

enum RawResponse {
    Session(Option<Session>),
    Timeline(bsky::feed::get_timeline::Output),
    Login,
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
        let raw_response = handle_request(request.clone(), &agent, &mut session).await;
        let response = convert_raw_response(raw_response, request);
        if tx.send(app::Response::Atp(response)).is_err() {
            break;
        }
    }

    Ok(())
}

#[instrument(
    name = "atp_request",
    err,
    skip(agent, session),
    fields(session = session.as_ref().map(|s| s.handle.as_str()))
)]
async fn handle_request(
    request: Request,
    agent: &AtpAgent<FileStore, ReqwestClient>,
    session: &mut Option<Session>,
) -> Result<RawResponse> {
    let res = match request {
        Request::GetSession => {
            let session = session.clone();
            RawResponse::Session(session)
        }
        Request::GetTimeline(params) => {
            let timeline = agent.api.app.bsky.feed.get_timeline(params).await?;
            RawResponse::Timeline(timeline)
        }
        Request::Login { ident, passwd } => {
            *session = Some(agent.login(ident, passwd).await?);
            RawResponse::Login
        }
    };
    Ok(res)
}

fn convert_raw_response(response: Result<RawResponse>, request: Request) -> Response {
    match response {
        Ok(RawResponse::Session(r)) => Response::Session(r),
        Ok(RawResponse::Timeline(r)) => Response::Timeline(Ok(r)),
        Ok(RawResponse::Login) => Response::Login(Ok(())),
        Err(e) => match request {
            Request::GetSession => unreachable!(),
            Request::GetTimeline(_) => Response::Timeline(Err(e)),
            Request::Login { .. } => Response::Login(Err(e)),
        },
    }
}

#[derive(Clone, Debug)]
struct FileStore(PathBuf);

impl FileStore {
    pub fn new() -> Result<Self> {
        Ok(Self(crate::utils::local_data_dir()?.join("session.json")))
    }

    fn open_file(&self) -> Result<File> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.0)?;
        Ok(file)
    }

    #[instrument(err)]
    fn get(&self) -> Result<Session> {
        let session = serde_json::from_reader(self.open_file()?)?;
        Ok(session)
    }

    #[instrument(err, skip(session))]
    fn set(&self, session: Session) -> Result<()> {
        serde_json::to_writer(self.open_file()?, &session)?;
        Ok(())
    }

    #[instrument(err)]
    async fn clear(&self) -> Result<()> {
        tokio::fs::remove_file(&self.0).await?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for FileStore {
    async fn get_session(&self) -> Option<Session> {
        self.get().ok()
    }

    async fn set_session(&self, session: Session) {
        self.set(session).ok();
    }

    async fn clear_session(&self) {
        self.clear().await.ok();
    }
}
