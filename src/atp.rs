mod response;
mod session;

use std::sync::Arc;

use anyhow::{Context, Result};
use atrium_api::{agent::AtpAgent, app::bsky};
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use tracing::instrument;

pub use self::response::Response;
use self::session::FileStore;

pub struct Atp {
    agent: Agent,
}

type Agent = Arc<AtpAgent<FileStore, ReqwestClient>>;

impl Atp {
    pub fn new() -> Result<Self> {
        const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let xrpc_client = ReqwestClientBuilder::new("https://bsky.social")
            .client(reqwest::Client::builder().user_agent(USER_AGENT).build()?)
            .build();
        let session_store = FileStore::new()?;
        Ok(Self {
            agent: Arc::new(AtpAgent::new(xrpc_client, session_store)),
        })
    }

    fn agent(&self) -> Agent {
        Arc::clone(&self.agent)
    }

    pub fn get_timeline(
        &self,
        params: bsky::feed::get_timeline::Parameters,
    ) -> Response<GetTimelineResult> {
        Response::new(get_timeline(self.agent(), params))
    }

    pub fn login(&self, ident: String, passwd: String) -> Response<LoginResult> {
        Response::new(login(self.agent(), ident, passwd))
    }

    pub fn resume_session(&self) -> Response<ResumeSessionResult> {
        Response::new(resume_session(self.agent()))
    }
}

pub type GetTimelineResult = Result<bsky::feed::get_timeline::Output>;

#[instrument(ret, err, skip_all)]
async fn get_timeline(
    agent: Agent,
    params: bsky::feed::get_timeline::Parameters,
) -> GetTimelineResult {
    let timeline = agent.api.app.bsky.feed.get_timeline(params).await?;
    Ok(timeline)
}

pub type LoginResult = Result<()>;

#[instrument(ret, err, skip_all)]
async fn login(agent: Agent, ident: String, passwd: String) -> LoginResult {
    agent.login(ident, passwd).await?;
    Ok(())
}

pub type ResumeSessionResult = Result<()>;

#[instrument(ret, err, skip_all)]
async fn resume_session(agent: Agent) -> ResumeSessionResult {
    let session = agent
        .get_session()
        .await
        .context("cannot find an existing session")?;
    agent.resume_session(session).await?;
    Ok(())
}
