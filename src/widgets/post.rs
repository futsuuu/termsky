use std::sync::{Arc, Mutex};

use atrium_api::{app::bsky::feed::defs::FeedViewPost, records::Record};
use ratatui::{prelude::*, widgets::*};

#[derive(Clone, Debug)]
pub struct Posts {
    posts: Vec<(Post, Arc<Mutex<PostState>>)>,
    scrolled_posts: usize,
}

impl Posts {
    pub fn new() -> Self {
        Self {
            posts: Vec::new(),
            scrolled_posts: 0,
        }
    }

    pub fn add_post(&mut self, post: Post, new: bool) {
        let post_state = Arc::new(Mutex::new(PostState { height: 0 }));
        if new {
            self.posts.insert(0, (post, post_state));
        } else {
            self.posts.push((post, post_state));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.posts.is_empty()
    }
}

impl WidgetRef for Posts {
    fn render_ref(&self, mut area: Rect, buf: &mut Buffer) {
        for (post, post_state) in self.posts.iter().skip(self.scrolled_posts) {
            let mut post_state = post_state.lock().unwrap();
            post.render_ref(area, buf, &mut post_state);
            area.y += post_state.height;
            area.height = area.height.saturating_sub(post_state.height);
            if area.height == 0 {
                break;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Post {
    /// display name or handle
    name: String,
    /// handle if display name not set
    second_name: Option<String>,
    /// post content
    text: String,
}

#[derive(Clone, Debug)]
pub struct PostState {
    height: u16,
}

impl StatefulWidgetRef for Post {
    type State = PostState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let content = Paragraph::new(self.text.as_str()).wrap(Wrap { trim: false });
        let header = Paragraph::new({
            let mut spans = vec![self.name.as_str().bold()];
            if let Some(second_name) = &self.second_name {
                spans.append(&mut vec!["  ".into(), second_name.as_str().dim().italic()]);
            }
            Line::from(spans)
        })
        .block(Block::new().padding(Padding::vertical(1)));

        let [header_area, content_area] = if state.height == 0 {
            let content_height = content.line_count(area.width) as u16;
            let areas =
                Layout::vertical([Constraint::Length(3), Constraint::Length(content_height)])
                    .areas(area);
            *state = PostState {
                height: areas.iter().map(|a| a.height).sum(),
            };
            areas
        } else {
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(Rect {
                height: state.height,
                ..area
            })
        };

        header.render(header_area, buf);
        content.render(content_area, buf);
    }
}

impl From<FeedViewPost> for Post {
    fn from(value: FeedViewPost) -> Self {
        let author = &value.post.author;
        let handle = format!("@{}", author.handle.as_str());
        let (name, second_name) = match &author.display_name {
            Some(display_name) => (display_name.clone(), Some(handle)),
            None => (handle, None),
        };
        Self {
            name,
            second_name,
            text: match &value.post.record {
                Record::AppBskyFeedPost(rec) => rec.text.clone(),
                _ => String::from("unimplemented!"),
            },
        }
    }
}
