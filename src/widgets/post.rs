use atrium_api::{
    app::bsky::{self, feed::defs::FeedViewPost},
    records::Record,
};
use ratatui::{prelude::*, widgets::*};

use super::{LazyBuffer, LazyWidget};

#[derive(Clone, Debug)]
pub struct Posts {
    posts: Vec<Post>,
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
        if new {
            self.posts.insert(0, post);
        } else {
            self.posts.push(post);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.posts.is_empty()
    }
}

impl WidgetRef for Posts {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut lbuf = LazyBuffer::new();
        for post in self.posts.iter().skip(self.scrolled_posts) {
            let rendered = lbuf.rendered_area();
            let space = Rect {
                y: area.y + rendered.height,
                height: area.height.saturating_sub(rendered.height),
                ..area
            };
            if space.height == 0 {
                break;
            }
            post.render_lazy(space, &mut lbuf);
        }
        lbuf.render_ref(area, buf);
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

impl From<FeedViewPost> for Post {
    fn from(value: FeedViewPost) -> Self {
        let post = &value.post;
        Self {
            author: post.author.clone().into(),
            content: match &post.record {
                Record::AppBskyFeedPost(rec) => rec.text.clone(),
                _ => String::from("unimplemented!"),
            },
            likes: post.like_count.unwrap_or(0) as u64,
            replies: post.reply_count.unwrap_or(0) as u64,
            reposts: post.repost_count.unwrap_or(0) as u64,
            reposted_by: value.reason.map(|r| match r {
                bsky::feed::defs::FeedViewPostReasonEnum::ReasonRepost(repost) => repost.by.into(),
            }),
        }
    }
}

impl From<bsky::actor::defs::ProfileViewBasic> for Account {
    fn from(value: bsky::actor::defs::ProfileViewBasic) -> Self {
        let handle = format!("@{}", value.handle.as_str());
        match value.display_name {
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

impl<'a> LazyWidget<'a> for &'a Post {
    fn render_lazy(self, area: Rect, buf: &mut LazyBuffer<'a>) {
        let wrapped_content = textwrap::wrap(self.content.as_str(), area.width as usize);
        let [repost_info_area, header_area, content_area, footer_area] = Layout::vertical([
            Constraint::Length(if self.reposted_by.is_some() { 1 } else { 0 }),
            Constraint::Length(2),
            Constraint::Length(wrapped_content.len() as u16),
            Constraint::Length(3),
        ])
        .areas(Rect {
            height: u16::MAX,
            ..area
        });

        if let Some(reposted_by) = &self.reposted_by {
            Span::from(format!("  Reposted by {}", reposted_by.name))
                .render_lazy(repost_info_area, buf);
        }
        Paragraph::new({
            let mut spans = vec![self.author.name.clone().bold()];
            if let Some(opt_name) = &self.author.opt_name {
                spans.append(&mut vec!["  ".into(), opt_name.clone().dim().italic()]);
            }
            Line::from(spans)
        })
        .block(Block::new().padding(Padding::bottom(1)))
        .render_lazy(header_area, buf);
        Paragraph::new(
            wrapped_content
                .iter()
                .map(|s| s.to_string())
                .map(Line::from)
                .collect::<Vec<_>>(),
        )
        .render_lazy(content_area, buf);
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
        .render_lazy(footer_area, buf);
    }
}
