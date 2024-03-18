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

pub enum Response {
    Session(Option<Session>),
    Timeline(Result<bsky::feed::get_timeline::Output>),
    Login(Result<()>),
}

#[derive(Debug)]
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
    let xrpc_client = ReqwestClientBuilder::new("https://bsky.social")
        .client(reqwest::Client::builder().user_agent(USER_AGENT).build()?)
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

    event!(Level::DEBUG, "stop handler: channel closed");
    Ok(())
}

#[instrument(
    name = "atp_handler",
    err,
    ret,
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
