use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hasher},
    ops,
    rc::Rc,
};

use atrium_api::{app::bsky::feed::defs::FeedViewPost, records::Record};
use ratatui::{prelude::*, widgets::*};

type PostID = u64;

#[derive(Clone, Debug)]
pub struct Posts {
    posts: Vec<Post>,
}

pub struct PostsState {
    post_states: HashMap<PostID, Rc<PostState>>,
    scrolled_posts: usize,
}

impl Posts {
    pub fn new() -> Self {
        Self { posts: Vec::new() }
    }
}

impl StatefulWidgetRef for Posts {
    type State = PostsState;

    fn render_ref(&self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        for (_i, post) in self.posts.iter().enumerate().skip(state.scrolled_posts) {
            let mut post_state = state.post_states.get(&post.id).cloned();
            post.render_ref(area, buf, &mut post_state);
            let post_state = post_state.unwrap();
            area.y += post_state.height;
            area.height = area.height.saturating_sub(post_state.height);
            if area.height == 0 {
                break;
            }
        }
    }
}

impl ops::Deref for Posts {
    type Target = Vec<Post>;
    fn deref(&self) -> &Self::Target {
        &self.posts
    }
}

impl ops::DerefMut for Posts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.posts
    }
}

#[derive(Clone, Debug)]
pub struct Post {
    /// hash of CID
    id: PostID,
    /// display name or handle
    name: String,
    /// handle if display name not set
    second_name: Option<String>,
    /// post content
    text: String,
}

pub struct PostState {
    height: u16,
}

impl StatefulWidgetRef for Post {
    type State = Option<Rc<PostState>>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let paragraph = Paragraph::new(self.text.as_str()).wrap(Wrap { trim: false });
        let p_height = paragraph.line_count(area.width) as u16;
        let [header, content] =
            Layout::vertical([Constraint::Length(3), Constraint::Length(p_height)]).areas(area);
        *state = Some(Rc::new(PostState {
            height: header.height + content.height,
        }));
        self.render_header(header, buf);
        paragraph.render(content, buf);
    }
}

impl Post {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new({
            let mut spans = vec![self.name.as_str().bold()];
            if let Some(second_name) = &self.second_name {
                spans.append(&mut vec!["  ".into(), second_name.as_str().dim().italic()]);
            }
            Line::from(spans)
        })
        .block(Block::new().padding(Padding::vertical(1)))
        .render(area, buf);
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
            id: {
                let mut hasher = DefaultHasher::new();
                hasher.write(value.post.cid.as_ref().to_bytes().as_slice());
                hasher.finish()
            },
            name,
            second_name,
            text: match &value.post.record {
                Record::AppBskyFeedPost(rec) => rec.text.clone(),
                _ => String::from("unimplemented!"),
            },
        }
    }
}
