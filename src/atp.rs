mod session;

use std::fmt;

use anyhow::Result;
use atrium_api::{
    agent::{store::SessionStore, AtpAgent, Session},
    app::bsky,
};
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use tokio::sync::mpsc;
use tracing::{event, instrument, Level};

use crate::app;
use session::FileStore;

#[derive(Clone)]
pub enum Request {
    GetSession,
    GetTimeline(bsky::feed::get_timeline::Parameters),
    Login { ident: String, passwd: String },
}

pub enum Response {
    Session(Box<Option<Session>>),
    Timeline(Result<bsky::feed::get_timeline::Output>),
    Login(Result<()>),
}

#[derive(Debug)]
enum RawResponse {
    Session(Box<Option<Session>>),
    Timeline(bsky::feed::get_timeline::Output),
    Login,
}

pub struct Atp {
    req_rx: mpsc::UnboundedReceiver<Request>,
    res_tx: mpsc::UnboundedSender<app::Response>,
    agent: AtpAgent<FileStore, ReqwestClient>,
    session: Option<Session>,
    session_store: FileStore,
}

impl Atp {
    pub fn new(
        req_rx: mpsc::UnboundedReceiver<Request>,
        res_tx: mpsc::UnboundedSender<app::Response>,
    ) -> Result<Self> {
        const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let xrpc_client = ReqwestClientBuilder::new("https://bsky.social")
            .client(reqwest::Client::builder().user_agent(USER_AGENT).build()?)
            .build();
        let session_store = FileStore::new()?;
        let agent = AtpAgent::new(xrpc_client, session_store.clone());
        Ok(Self {
            req_rx,
            res_tx,
            agent,
            session: None,
            session_store,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        while let Some(req) = self.req_rx.recv().await {
            let r_res = self.handle_request(req.clone()).await;
            let res = convert_raw_response(r_res, req);
            if self.res_tx.send(app::Response::Atp(res)).is_err() {
                break;
            }
        }
        event!(Level::DEBUG, "stop handler: channel closed");
        Ok(())
    }

    #[instrument(name = "atp", err, ret, skip(self), fields(session = self.handle()))]
    async fn handle_request(&mut self, request: Request) -> Result<RawResponse> {
        let res = match request {
            Request::GetSession => {
                let session = self.get_session().await;
                RawResponse::Session(Box::new(session))
            }
            Request::GetTimeline(params) => {
                let timeline = self.agent.api.app.bsky.feed.get_timeline(params).await?;
                RawResponse::Timeline(timeline)
            }
            Request::Login { ident, passwd } => {
                self.session = Some(self.agent.login(ident, passwd).await?);
                RawResponse::Login
            }
        };
        Ok(res)
    }

    async fn get_session(&mut self) -> Option<Session> {
        if let Some(session) = &self.session {
            return Some(session.clone());
        }
        let session = self.session_store.get_session().await?;
        self.agent.resume_session(session.clone()).await.ok()?;
        self.session = Some(session.clone());
        self.session.clone()
    }

    fn handle(&self) -> Option<&str> {
        self.session.as_ref().map(|s| s.handle.as_str())
    }
}

fn convert_raw_response(response: Result<RawResponse>, request: Request) -> Response {
    match response {
        Ok(res) => match res {
            RawResponse::Session(r) => Response::Session(r),
            RawResponse::Timeline(r) => Response::Timeline(Ok(r)),
            RawResponse::Login => Response::Login(Ok(())),
        },
        Err(e) => match request {
            Request::GetSession => unreachable!(),
            Request::GetTimeline(_) => Response::Timeline(Err(e)),
            Request::Login { .. } => Response::Login(Err(e)),
        },
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Request::GetSession => f.write_str("GetSession"),
            Request::GetTimeline(params) => f.debug_tuple("GetTimeline").field(&params).finish(),
            Request::Login {
                ident,
                passwd: _passwd,
            } => f
                .debug_struct("Login")
                .field("ident", &ident)
                .field("passwd", &"***")
                .finish(),
        }
    }
}
