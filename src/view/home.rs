use std::cell::RefCell;

use atrium_api::app::bsky;
use ratatui::{prelude::*, widgets::*};

use crate::widgets::{Posts, PostsState};

#[derive(Debug)]
pub struct Home {
    posts: Posts,
    posts_state: RefCell<PostsState>,
    /// Used to get old posts
    post_cursor: Option<String>,
    /// true when waiting the response
    waiting: bool,
}

impl Home {
    pub fn new() -> Self {
        Self {
            posts: Posts::new(),
            posts_state: RefCell::new(PostsState::new()),
            post_cursor: None,
            waiting: false,
        }
    }

    pub fn get_timeline_params(&mut self) -> Option<bsky::feed::get_timeline::Parameters> {
        if self.posts_state.borrow().blank_height.is_none() || self.waiting {
            return None;
        }
        self.waiting = true;
        Some(bsky::feed::get_timeline::Parameters {
            algorithm: None,
            cursor: self.post_cursor.clone(),
            limit: 15.try_into().ok(),
        })
    }

    pub fn recv_timeline(&mut self, timeline: bsky::feed::get_timeline::Output) {
        self.waiting = false;
        self.post_cursor = timeline.cursor;
        for post in timeline.feed {
            self.posts.add_post(post, false);
        }
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
        self.posts.render_ref(area, buf, &mut self.posts_state.borrow_mut());
    }
}
