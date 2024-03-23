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
    author: Author,
    content: String,
    likes: u64,
    replies: u64,
    reposts: u64,
}

#[derive(Clone, Debug)]
struct Author {
    name: String,
    opt: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PostState {
    height: u16,
}

impl StatefulWidgetRef for Post {
    type State = PostState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let content = Paragraph::new(self.content.as_str()).wrap(Wrap { trim: false });

        let header_constraint = Constraint::Length(2);
        let footer_constraint = Constraint::Length(3);
        let [header_area, content_area, footer_area] = if state.height == 0 {
            let content_height = content.line_count(area.width) as u16;
            let areas = Layout::vertical([
                header_constraint,
                Constraint::Length(content_height),
                footer_constraint,
            ])
            .areas(area);
            *state = PostState {
                height: areas.iter().map(|a| a.height).sum(),
            };
            areas
        } else {
            Layout::vertical([header_constraint, Constraint::Fill(1), footer_constraint]).areas(
                Rect {
                    height: state.height,
                    ..area
                },
            )
        };

        Paragraph::new({
            let mut spans = vec![self.author.name.as_str().bold()];
            if let Some(opt) = &self.author.opt {
                spans.append(&mut vec!["  ".into(), opt.as_str().dim().italic()]);
            }
            Line::from(spans)
        })
        .block(Block::new().padding(Padding::bottom(1)))
        .render(header_area, buf);
        content.render(content_area, buf);
        Paragraph::new(format!(
            " {}    {}   ♥ {}",
            self.replies, self.reposts, self.likes
        ))
        .block(
            Block::new()
                .padding(Padding::top(1))
                .borders(Borders::BOTTOM)
                .border_style(Style::new().dim()),
        )
        .render(footer_area, buf);
    }
}

impl From<FeedViewPost> for Post {
    fn from(value: FeedViewPost) -> Self {
        let post = &value.post;
        let author = {
            let author = &post.author;
            let handle = format!("@{}", author.handle.as_str());
            let (name, opt) = match &author.display_name {
                Some(display_name) => (display_name.clone(), Some(handle)),
                None => (handle, None),
            };
            Author { name, opt }
        };
        Self {
            author,
            content: match &post.record {
                Record::AppBskyFeedPost(rec) => rec.text.clone(),
                _ => String::from("unimplemented!"),
            },
            likes: post.like_count.unwrap_or(0) as u64,
            replies: post.reply_count.unwrap_or(0) as u64,
            reposts: post.repost_count.unwrap_or(0) as u64,
        }
    }
}
