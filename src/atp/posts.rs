use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use atrium_api::app::bsky::{
    self,
    feed::{get_posts, get_timeline},
};
use indexmap::IndexMap;

use super::Agent;

pub type GetTimelineResult = anyhow::Result<IndexMap<String, Post>>;
pub type GetPostsResult = anyhow::Result<IndexMap<String, Post>>;

pub(super) struct Posts {
    posts: Mutex<IndexMap<String, PostInner>>,
    cursors: Mutex<HashMap<FeedKind, Option<String>>>,
}

impl Default for Posts {
    fn default() -> Self {
        Self {
            posts: Mutex::new(IndexMap::new()),
            cursors: Mutex::new(HashMap::new()),
        }
    }
}

impl Posts {
    pub async fn get_timeline(&self, agent: Agent) -> GetTimelineResult {
        let params = get_timeline::Parameters {
            cursor: self.get_cursor(&FeedKind::Timeline),
            algorithm: None,
            limit: 15.try_into().ok(),
        };
        let output = agent.api.app.bsky.feed.get_timeline(params).await?;
        self.set_cursor(FeedKind::Timeline, output.cursor);
        let mut posts = self.posts.lock().unwrap();
        let result = output
            .feed
            .into_iter()
            .map(|feed_view_post| {
                let reasons = match feed_view_post.reason {
                    Some(atrium_api::types::Union::Refs(reason)) => vec![reason],
                    _ => vec![],
                };
                insert_post_view(
                    &mut posts,
                    feed_view_post.post,
                    Some((FeedKind::Timeline, reasons)),
                )
            })
            .collect();
        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn get_posts(&self, agent: Agent, uris: Vec<String>) -> GetPostsResult {
        let params = get_posts::Parameters { uris };
        let output = agent.api.app.bsky.feed.get_posts(params).await?;

        let mut posts = self.posts.lock().unwrap();
        let result = output
            .posts
            .into_iter()
            .map(|post_view| insert_post_view(&mut posts, post_view, None))
            .collect();
        Ok(result)
    }

    fn get_cursor(&self, kind: &FeedKind) -> Option<String> {
        self.cursors.lock().unwrap().get(kind)?.clone()
    }

    fn set_cursor(&self, kind: FeedKind, cursor: Option<String>) {
        tracing::debug!("update cursor: {cursor:?}");
        self.cursors.lock().unwrap().insert(kind, cursor);
    }
}

fn insert_post_view(
    posts: &mut IndexMap<String, PostInner>,
    post_view: bsky::feed::defs::PostView,
    kind_reasons: Option<(FeedKind, Vec<bsky::feed::defs::FeedViewPostReasonRefs>)>,
) -> (String, Post) {
    let uri = post_view.uri.clone();
    tracing::trace!("receive post {uri}");

    let kind = kind_reasons.as_ref().map(|a| a.0.clone());

    let post_inner = if let Some(post_inner) = posts.get(&uri) {
        *post_inner.post.lock().unwrap() = post_view;
        if let Some((kind, reasons)) = kind_reasons {
            post_inner
                .reasons
                .lock()
                .unwrap()
                .entry(kind.clone())
                .or_insert(Vec::new())
                .extend(reasons);
        }
        post_inner.clone()
    } else {
        let post_inner = if let Some(kind_reasons) = kind_reasons {
            PostInner::new(post_view, [kind_reasons])
        } else {
            PostInner::from(post_view)
        };
        posts.insert(uri.clone(), post_inner.clone());
        post_inner
    };

    let post = Post {
        inner: post_inner,
        kind,
    };
    (uri, post)
}

#[derive(Clone, Debug)]
pub struct Post {
    inner: PostInner,
    kind: Option<FeedKind>,
}

impl Post {
    pub fn post(&self) -> std::sync::MutexGuard<'_, bsky::feed::defs::PostView> {
        self.inner.post.lock().unwrap()
    }

    pub fn reasons(&self) -> Vec<bsky::feed::defs::FeedViewPostReasonRefs> {
        let Some(kind) = &self.kind else {
            return Vec::new();
        };
        if let Some(reasons) = self.inner.reasons.lock().unwrap().get(kind) {
            return reasons.clone();
        }
        Vec::new()
    }
}

#[derive(Clone, Debug)]
struct PostInner {
    post: Arc<Mutex<bsky::feed::defs::PostView>>,
    reasons: Arc<Mutex<HashMap<FeedKind, Vec<bsky::feed::defs::FeedViewPostReasonRefs>>>>,
}

impl PostInner {
    fn new(
        post: bsky::feed::defs::PostView,
        reason: impl Into<HashMap<FeedKind, Vec<bsky::feed::defs::FeedViewPostReasonRefs>>>,
    ) -> Self {
        Self {
            post: Arc::new(Mutex::new(post)),
            reasons: Arc::new(Mutex::new(reason.into())),
        }
    }
}

impl From<bsky::feed::defs::PostView> for PostInner {
    fn from(post: bsky::feed::defs::PostView) -> Self {
        Self::new(post, [])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FeedKind {
    #[allow(dead_code)]
    Feed(String),
    Timeline,
}
