use atrium_api::app::bsky::feed::defs::FeedViewPost;
use ratatui::{prelude::*, widgets::*};

use crate::widgets::{Post, Posts, PostsState};

#[derive(Clone, Debug)]
pub struct Home {
    posts: Posts,
    /// Used to get new posts
    post_cursor: Option<String>,
    /// true when waiting the response
    waiting: bool,
}

pub struct HomeState {
    posts: PostsState,
}

impl Home {
    pub fn new() -> Self {
        Self {
            posts: Posts::new(),
            post_cursor: None,
            waiting: false,
        }
    }

    pub fn wait_response(&mut self) {
        self.waiting = true;
    }

    pub fn add_received_post(&mut self, post: FeedViewPost, new: bool) {
        self.waiting = false;
        let post = Post::from(post);
        if new {
            self.posts.insert(0, post);
        } else {
            self.posts.push(post);
        }
    }

    pub fn new_posts_required(&self) -> bool {
        self.posts.is_empty() && !self.waiting
    }
}

impl Widget for &Home {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(post) = self.posts.get(0) {
            post.render_ref(area, buf, &mut None);
        }
    }
}
