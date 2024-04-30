use std::cell::RefCell;

use atrium_api::app::bsky;
use ratatui::{prelude::*, widgets::*};

use crate::{
    prelude::*,
    widgets::{atoms::Spinner, Posts, PostsState, Tab, Tabs},
};

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
        let [tabs_area, _, posts_area, _] = Layout::horizontal([
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(6),
            Constraint::Fill(2),
        ])
        .horizontal_margin(1)
        .areas(area);

        // tabs
        Tabs {
            tabs: vec![Tab::new("1. Login"), Tab::new("2. Home")],
            selected: 1,
        }
        .render(tabs_area, buf);

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

impl AppHandler for Home {
    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {
        if let Some(params) = self.get_timeline_params() {
            app.send(AtpRequest::GetTimeline(params));
        }
        if let TuiEvent::Key(key_event) = ev {
            if key_event.code == KeyCode::Esc {
                app.exit();
                return;
            }
            if key_event.code == KeyCode::Char('k') {
                self.scroll_up();
            }
            if key_event.code == KeyCode::Char('j') {
                self.scroll_down();
            }
        }
    }

    fn atp_response(&mut self, _app: &mut App, res: AtpResponse) {
        if let AtpResponse::Timeline(Ok(timeline)) = res {
            self.recv_timeline(timeline);
        }
    }
}
