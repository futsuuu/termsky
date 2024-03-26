use std::sync::{Arc, Mutex};

use atrium_api::{
    app::bsky::feed::defs::{FeedViewPost, FeedViewPostReasonEnum},
    records::Record,
};
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

    pub fn add_post(&mut self, post: FeedViewPost, new: bool) {
        let post = post.into();
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
struct Post {
    author: Account,
    content: String,
    likes: u64,
    replies: u64,
    reposts: u64,
    reposted_by: Option<Account>,
}

#[derive(Clone, Debug)]
struct Account {
    name: String,
    opt_name: Option<String>,
}

#[derive(Debug)]
struct PostState {
    height: u16,
}

impl Account {
    fn new(display_name: Option<String>, handle: &atrium_api::types::string::Handle) -> Self {
        let handle = format!("@{}", handle.as_str());
        match display_name {
            Some(display_name) => Self {
                name: display_name,
                opt_name: Some(handle),
            },
            None => Self {
                name: handle,
                opt_name: None,
            },
        }
    }
}

impl StatefulWidgetRef for Post {
    type State = PostState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let wrapped_content = textwrap::wrap(self.content.as_str(), area.width as usize);
        let [repost_info_area, header_area, content_area, footer_area] = {
            let content_height = wrapped_content.len() as u16;
            let areas = Layout::vertical([
                Constraint::Length(if self.reposted_by.is_some() { 1 } else { 0 }),
                Constraint::Length(2),
                Constraint::Length(content_height),
                Constraint::Length(3),
            ])
            .areas(area);
            *state = PostState {
                height: areas.iter().map(|a| a.height).sum(),
            };
            areas
        };

        if let Some(reposted_by) = &self.reposted_by {
            Paragraph::new(format!("  Reposted by {}", reposted_by.name))
                .render(repost_info_area, buf);
        }
        Paragraph::new({
            let mut spans = vec![self.author.name.as_str().bold()];
            if let Some(opt_name) = &self.author.opt_name {
                spans.append(&mut vec!["  ".into(), opt_name.as_str().dim().italic()]);
            }
            Line::from(spans)
        })
        .block(Block::new().padding(Padding::bottom(1)))
        .render(header_area, buf);
        Paragraph::new(
            wrapped_content
                .iter()
                .map(|s| s.to_string())
                .map(Line::from)
                .collect::<Vec<_>>(),
        )
        .render(content_area, buf);
        Paragraph::new(format!(
            " {}    {}   ♥ {}",
            self.replies, self.reposts, self.likes
        ))
        .block(
            Block::new()
                .padding(Padding::top(1))
                .borders(Borders::BOTTOM)
                .border_style(Style::new().blue().dim()),
        )
        .render(footer_area, buf);
    }
}

impl From<FeedViewPost> for Post {
    fn from(value: FeedViewPost) -> Self {
        let post = &value.post;
        Self {
            author: {
                let author = &post.author;
                Account::new(author.display_name.clone(), &author.handle)
            },
            content: match &post.record {
                Record::AppBskyFeedPost(rec) => rec.text.clone(),
                _ => String::from("unimplemented!"),
            },
            likes: post.like_count.unwrap_or(0) as u64,
            replies: post.reply_count.unwrap_or(0) as u64,
            reposts: post.repost_count.unwrap_or(0) as u64,
            reposted_by: value.reason.map(|r| match r {
                FeedViewPostReasonEnum::ReasonRepost(repost) => {
                    Account::new(repost.by.display_name, &repost.by.handle)
                }
            }),
        }
    }
}
