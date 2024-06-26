use std::cell::RefCell;

use atrium_api::app::bsky;
use ratatui::{prelude::*, widgets::*};

use crate::{
    atp::Response,
    prelude::*,
    widgets::{atoms::Spinner, Posts, PostsState},
};

#[derive(Debug, Default)]
pub struct Home {
    posts: Posts,
    posts_state: RefCell<PostsState>,
    response: Response<crate::atp::GetTimelineResult>,
    /// Used to get old posts
    post_cursor: Option<String>,
}

impl Home {
    pub fn get_timeline_params(&self) -> bsky::feed::get_timeline::Parameters {
        bsky::feed::get_timeline::Parameters {
            algorithm: None,
            cursor: self.post_cursor.clone(),
            limit: 15.try_into().ok(),
        }
    }

    pub fn recv_timeline(&mut self, timeline: bsky::feed::get_timeline::Output) {
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
        let [_, posts_area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(5),
            Constraint::Fill(1),
        ])
        .areas(area);

        // posts
        let mut posts_state = self.posts_state.borrow_mut();
        self.posts.render_ref(posts_area, buf, &mut posts_state);

        // spinner
        let Some(blank_height) = posts_state.blank_height else {
            return;
        };
        let blank_area = Rect {
            height: blank_height,
            y: posts_area.height - blank_height,
            ..posts_area
        };
        let [_, spinner_area] = Layout::vertical([
            Constraint::Length(14_u16.saturating_sub(blank_area.height)),
            Constraint::Fill(1),
        ])
        .areas(blank_area);
        Spinner::new().render_ref(spinner_area, buf);
    }
}

impl crate::app::EventHandler for Home {
    fn on_render(&mut self, app: &mut App) {
        if self.posts_state.borrow().blank_height.is_some() && self.response.is_empty() {
            self.response = app.atp.get_timeline(self.get_timeline_params());
        }

        if let Some(Ok(timeline)) = self.response.take_data() {
            self.recv_timeline(timeline);
        }
    }

    fn on_key(&mut self, ev: crossterm::event::KeyEvent, app: &mut App) {
        if ev.code == KeyCode::Esc {
            app.exit();
            return;
        }
        if ev.code == KeyCode::Char('k') {
            self.scroll_up();
        }
        if ev.code == KeyCode::Char('j') {
            self.scroll_down();
        }
    }
}
