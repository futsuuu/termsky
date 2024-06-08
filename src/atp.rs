mod notifications;
mod posts;
mod response;
mod session;
pub mod types;

use std::sync::Arc;

use anyhow::{Context, Result};
use atrium_api::agent::AtpAgent;
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use tracing::instrument;

use self::notifications::Notifications;
pub use self::notifications::{GetNotificationsError, GetNotificationsResult, Notification};
use self::posts::Posts;
pub use self::posts::{GetTimelineResult, Post};
pub use self::response::Response;
use self::session::FileStore;

pub struct Atp {
    agent: Agent,
    posts: Arc<Posts>,
    notifications: Arc<Notifications>,
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
            posts: Arc::new(Posts::default()),
            notifications: Arc::new(Notifications::default()),
        })
    }

    #[inline]
    fn agent(&self) -> Agent {
        Arc::clone(&self.agent)
    }

    pub fn get_timeline(&self) -> Response<GetTimelineResult> {
        let agent = self.agent();
        let posts = Arc::clone(&self.posts);
        Response::new(async move { posts.get_timeline(agent).await })
    }

    pub fn get_notifications(&self) -> Response<GetNotificationsResult> {
        let agent = self.agent();
        let notifications = Arc::clone(&self.notifications);
        Response::new(async move { notifications.get_old(agent).await })
    }

    pub fn login(&self, ident: String, passwd: String) -> Response<LoginResult> {
        Response::new(login(self.agent(), ident, passwd))
    }

    pub fn resume_session(&self) -> Response<ResumeSessionResult> {
        Response::new(resume_session(self.agent()))
    }
}

pub type LoginResult = Result<()>;

#[instrument(ret, err, skip(agent, passwd))]
async fn login(agent: Agent, ident: String, passwd: String) -> LoginResult {
    agent.login(ident, passwd).await?;
    Ok(())
}

pub type ResumeSessionResult = Result<()>;

#[instrument(ret, err, skip(agent))]
async fn resume_session(agent: Agent) -> ResumeSessionResult {
    let session = agent
        .get_session()
        .await
        .context("cannot find an existing session")?;
    agent.resume_session(session).await?;
    Ok(())
}
