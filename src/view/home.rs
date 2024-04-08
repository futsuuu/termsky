use atrium_api::app::bsky::feed::defs::FeedViewPost;
use ratatui::{prelude::*, widgets::*};

use crate::widgets::Posts;

#[derive(Debug)]
pub struct Home {
    posts: Posts,
    /// Used to get old posts
    post_cursor: Option<String>,
    /// true when waiting the response
    waiting: bool,
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
        self.posts.add_post(post, new);
    }

    pub fn new_posts_required(&self) -> bool {
        self.posts.is_empty() && !self.waiting
    }

    pub fn scroll_up(&mut self) {
        self.posts.scroll = self.posts.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.posts.scroll += 1;
    }
}

impl WidgetRef for Home {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(2),
            Constraint::Fill(5),
            Constraint::Fill(2),
        ])
        .areas(area);
        self.posts.render_ref(area, buf);
    }
}
