use atrium_api::{app::bsky::feed::defs::FeedViewPost, records::Record};
use ratatui::{prelude::*, widgets::*};

#[derive(Clone, Debug)]
pub struct Home {
    posts: Vec<Post>,
    /// Used to get new posts
    post_cursor: Option<String>,
    /// true when waiting the response
    waiting: bool,
}

impl Home {
    pub fn new() -> Self {
        Self {
            posts: Vec::new(),
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
            post.render(area, buf);
        }
    }
}

#[derive(Clone, Debug)]
struct Post {
    /// display name or handle
    name: String,
    /// handle if display name not set
    another_name: Option<String>,
    /// Post content
    text: String,
}

impl Widget for &Post {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, content] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);
        Paragraph::new({
            let mut spans = vec![self.name.as_str().bold()];
            if let Some(another_name) = &self.another_name {
                spans.append(&mut vec!["  ".into(), another_name.as_str().dim().italic()]);
            }
            Line::from(spans)
        })
        .block(Block::new().padding(Padding::vertical(1)))
        .render(top, buf);
        Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: false })
            .render(content, buf);
    }
}

impl From<FeedViewPost> for Post {
    fn from(value: FeedViewPost) -> Self {
        let author = &value.post.author;
        let handle = format!("@{}", author.handle.as_str());
        let (name, another_name) = match &author.display_name {
            Some(display_name) => (display_name.clone(), Some(handle)),
            None => (handle, None),
        };
        let text = match &value.post.record {
            Record::AppBskyFeedPost(rec) => rec.text.clone(),
            _ => String::from("unimplemented!"),
        };
        Self {
            name,
            another_name,
            text,
        }
    }
}
