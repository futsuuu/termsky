use atrium_api::{
    app::bsky::{self, feed::defs::FeedViewPost},
    records,
    types::Union,
};
use ratatui::{prelude::*, widgets::*};

use super::{Store, Storeable};

#[derive(Debug)]
pub struct Posts {
    posts: Vec<Post>,
    pub scroll: u16,
}

#[derive(Debug)]
pub struct PostsState {
    pub blank_height: Option<u16>,
}

impl Posts {
    pub fn new() -> Self {
        Self {
            posts: Vec::new(),
            scroll: 0,
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
}

impl PostsState {
    pub fn new() -> Self {
        Self { blank_height: None }
    }
}

impl StatefulWidgetRef for Posts {
    type State = PostsState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut store = Store::new().scroll_v(self.scroll as i32);
        for post in &self.posts {
            // post
            post.store(
                Rect {
                    y: store.stored_area().bottom(),
                    height: u16::MAX,
                    ..area
                },
                &mut store,
            );

            // border
            Block::new()
                .borders(Borders::TOP)
                .border_style(Style::new().blue().dim())
                .store(
                    Rect {
                        y: store.stored_area().bottom(),
                        height: 1,
                        ..area
                    },
                    &mut store,
                );
        }
        state.blank_height = (self.scroll + area.height).checked_sub(store.stored_area().height);
        store.render_ref(area, buf);
    }
}

nestify::nest! {
    #[derive(Debug)]*
    struct Post {
        author: struct Account {
            name: String,
            opt_name: Option<String>,
        },
        content: String,
        counts: Option<struct Counts {
            likes: u64,
            replies: u64,
            reposts: u64,
        }>,
        reposted_by: Option<Account>,
        embed: Option<enum Embed {
            Media(enum EmbedMedia {
                External(bsky::embed::external::ViewExternal),
                Image(Vec<struct EmbedImage {
                    alt: String,
                }>),
            }),
            Record(enum EmbedRecord {
                NotFound,
                Blocked,
                Post(Box<Post>),
                Unimplemented,
            }),
            RecordWithMedia(EmbedRecord, EmbedMedia),
            Unimplemented,
        }>,
    }
}

impl From<FeedViewPost> for Post {
    fn from(value: FeedViewPost) -> Self {
        let post = &value.post;
        Self {
            author: post.author.clone().into(),
            content: match &post.record {
                records::Record::Known(records::KnownRecord::AppBskyFeedPost(record)) => {
                    record.text.clone()
                }
                _ => String::from("unimplemented!"),
            },
            counts: Some(Counts {
                likes: post.like_count.unwrap_or(0) as u64,
                replies: post.reply_count.unwrap_or(0) as u64,
                reposts: post.repost_count.unwrap_or(0) as u64,
            }),
            reposted_by: match value.reason {
                Some(Union::Refs(bsky::feed::defs::FeedViewPostReasonRefs::ReasonRepost(
                    repost,
                ))) => Some(repost.by.into()),
                _ => None,
            },
            embed: match post.embed.clone() {
                Some(Union::Refs(embed)) => Some(embed.into()),
                _ => None,
            },
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

impl From<bsky::feed::defs::PostViewEmbedRefs> for Embed {
    fn from(value: bsky::feed::defs::PostViewEmbedRefs) -> Self {
        use bsky::feed::defs::PostViewEmbedRefs::*;
        match value {
            AppBskyEmbedExternalView(view) => EmbedMedia::from(view).into(),
            AppBskyEmbedImagesView(view) => EmbedMedia::from(view).into(),
            AppBskyEmbedRecordView(view) => view.into(),
            AppBskyEmbedRecordWithMediaView(view) => view.into(),
        }
    }
}

impl From<bsky::embed::record::ViewRecordEmbedsItem> for Embed {
    fn from(value: bsky::embed::record::ViewRecordEmbedsItem) -> Self {
        use bsky::embed::record::ViewRecordEmbedsItem::*;
        match value {
            AppBskyEmbedExternalView(view) => EmbedMedia::from(view).into(),
            AppBskyEmbedImagesView(view) => EmbedMedia::from(view).into(),
            AppBskyEmbedRecordView(view) => view.into(),
            AppBskyEmbedRecordWithMediaView(view) => view.into(),
        }
    }
}

impl From<Box<bsky::embed::record::View>> for Embed {
    fn from(value: Box<bsky::embed::record::View>) -> Self {
        match value.record {
            Union::Refs(refs) => Self::Record(refs.into()),
            Union::Unknown(_) => Self::Unimplemented,
        }
    }
}

impl From<EmbedMedia> for Embed {
    fn from(value: EmbedMedia) -> Self {
        Self::Media(value)
    }
}

impl From<Box<bsky::embed::record_with_media::View>> for Embed {
    fn from(value: Box<bsky::embed::record_with_media::View>) -> Self {
        let Union::Refs(record) = value.record.record else {
            return Self::Unimplemented;
        };
        let record = EmbedRecord::from(record);
        let Union::Refs(media) = value.media else {
            return Self::Record(record);
        };
        let media = EmbedMedia::from(media);
        Self::RecordWithMedia(record, media)
    }
}

impl From<bsky::embed::record::ViewRecordRefs> for EmbedRecord {
    fn from(value: bsky::embed::record::ViewRecordRefs) -> Self {
        use bsky::embed::record::ViewRecordRefs::*;
        match value {
            ViewNotFound(_) => Self::NotFound,
            ViewBlocked(_) => Self::Blocked,
            ViewRecord(record) => Self::Post(Box::new(Post {
                author: record.author.into(),
                content: match record.value {
                    records::Record::Known(records::KnownRecord::AppBskyFeedPost(record)) => {
                        record.text.clone()
                    }
                    _ => String::from("unimplemented!"),
                },
                embed: record
                    .embeds
                    .and_then(|e| {
                        e.into_iter()
                            .filter_map(|e| match e {
                                Union::Refs(e) => Some(e),
                                _ => None,
                            })
                            .next()
                    })
                    .map(Into::into),
                counts: None,
                reposted_by: None,
            })),
            _ => Self::Unimplemented,
        }
    }
}

impl From<bsky::embed::record_with_media::ViewMediaRefs> for EmbedMedia {
    fn from(value: bsky::embed::record_with_media::ViewMediaRefs) -> Self {
        use bsky::embed::record_with_media::ViewMediaRefs::*;
        match value {
            AppBskyEmbedImagesView(view) => view.into(),
            AppBskyEmbedExternalView(view) => view.into(),
        }
    }
}

impl From<Box<bsky::embed::external::View>> for EmbedMedia {
    fn from(value: Box<bsky::embed::external::View>) -> Self {
        Self::External(value.external)
    }
}

impl From<Box<bsky::embed::images::View>> for EmbedMedia {
    fn from(value: Box<bsky::embed::images::View>) -> Self {
        Self::Image(value.images.into_iter().map(Into::into).collect())
    }
}

impl From<bsky::embed::images::ViewImage> for EmbedImage {
    fn from(value: bsky::embed::images::ViewImage) -> Self {
        Self { alt: value.alt }
    }
}

impl<'a> Storeable<'a> for &'a Post {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        let wrapped_content = wrap(self.content.as_str(), area.width as usize);
        let embed_height = self
            .embed
            .as_ref()
            .map(|e| {
                let mut store = Store::new();
                e.store(area, &mut store);
                store.stored_area().height
            })
            .unwrap_or_default();
        let [repost_info_area, header_area, content_area, embed_area, footer_area] =
            Layout::vertical([
                Constraint::Length(if self.reposted_by.is_some() { 1 } else { 0 }),
                Constraint::Length(2),
                Constraint::Length(wrapped_content.len() as u16),
                Constraint::Length(embed_height),
                Constraint::Length(if self.counts.is_some() { 2 } else { 0 }),
            ])
            .areas(Rect {
                height: u16::MAX,
                ..area
            });

        if let Some(reposted_by) = &self.reposted_by {
            Span::from(format!("  Reposted by {}", reposted_by.name))
                .store(repost_info_area, store);
        }
        Paragraph::new({
            let mut spans = vec![self.author.name.clone().bold()];
            if let Some(opt_name) = &self.author.opt_name {
                spans.extend(["  ".into(), opt_name.clone().dim().italic()]);
            }
            Line::from(spans)
        })
        .block(Block::new().padding(Padding::bottom(1)))
        .store(header_area, store);
        Paragraph::new(wrapped_content).store(content_area, store);
        if let Some(embed) = &self.embed {
            embed.store(embed_area, store);
        }
        if let Some(counts) = &self.counts {
            Paragraph::new(format!(
                " {}    {}   ♥ {}",
                counts.replies, counts.reposts, counts.likes
            ))
            .block(Block::new().padding(Padding::top(1)))
            .store(footer_area, store);
        }
    }
}

impl<'a> Storeable<'a> for &'a Embed {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        match self {
            Embed::Record(record) => {
                record.store(area, store);
            }
            Embed::Media(media) => {
                media.store(area, store);
            }
            Embed::RecordWithMedia(record, media) => {
                media.store(area, store);
                record.store(
                    Rect {
                        y: store.stored_area().bottom(),
                        ..area
                    },
                    store,
                );
            }
            Embed::Unimplemented => {
                Paragraph::new("unimplemented!")
                    .block(embed_block())
                    .store(Rect { height: 3, ..area }, store);
            }
        }
    }
}

impl<'a> Storeable<'a> for &'a EmbedRecord {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        match self {
            EmbedRecord::Post(post) => {
                let block = embed_block();
                let post_area = block.inner(area);
                let mut s = Store::new();
                post.store(post_area, &mut s);
                let stored_area = s.stored_area();
                block.store(
                    Rect {
                        height: stored_area.height + 2,
                        ..area
                    },
                    store,
                );
                s.store(stored_area, store);
            }
            EmbedRecord::NotFound => {
                Paragraph::new("  Not Found")
                    .block(embed_block())
                    .store(Rect { height: 3, ..area }, store);
            }
            EmbedRecord::Blocked => {
                Paragraph::new("  Blocked")
                    .block(embed_block())
                    .store(Rect { height: 3, ..area }, store);
            }
            EmbedRecord::Unimplemented => {
                Paragraph::new("unimplemented!")
                    .block(embed_block())
                    .store(Rect { height: 3, ..area }, store);
            }
        }
    }
}

impl<'a> Storeable<'a> for &'a EmbedMedia {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        match self {
            EmbedMedia::External(external) => {
                let block = embed_block();
                let width = block.inner(area).width as usize;
                let title = {
                    let mut title: Vec<_> = wrap(&external.title, width)
                        .into_iter()
                        .take(3)
                        .map(|l| l.centered().style(Style::new().bold()))
                        .collect();
                    if !title.is_empty() {
                        let title_width = title.iter().map(Line::width).max().unwrap_or_default();
                        title.push(Line::from("▔".repeat(title_width)).centered())
                    }
                    title
                };
                let desc = {
                    let mut desc: Vec<_> = wrap(&external.description, width)
                        .into_iter()
                        .take(3)
                        .collect();
                    if !desc.is_empty() {
                        desc.push(Line::from(""));
                    }
                    desc
                };
                let uri = Line::from(external.uri.as_str()).style(Style::new().dim());

                let mut lines = Vec::new();
                lines.extend(title);
                lines.extend(desc);
                lines.push(uri);

                let height = lines.len() as u16 + 2; // inner + border
                Paragraph::new(lines)
                    .block(block)
                    .store(Rect { height, ..area }, store);
            }

            EmbedMedia::Image(images) => {
                let mut y = area.y;
                for image in images {
                    let block = embed_block();
                    let mut lines = vec![" ".magenta().into()];
                    lines.extend(wrap(&image.alt, block.inner(area).width as usize));

                    let height = lines.len() as u16 + 2;
                    Paragraph::new(lines)
                        .block(block)
                        .store(Rect { y, height, ..area }, store);
                    y += height;
                }
            }
        }
    }
}

fn wrap<'a>(text: &'a str, opts: impl Into<textwrap::Options<'a>>) -> Vec<Line<'a>> {
    if text.is_empty() {
        return Vec::new();
    }
    textwrap::wrap(text, opts)
        .iter()
        .map(|s| Line::from(s.to_string()))
        .collect()
}

fn embed_block() -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dim())
        .padding(Padding::horizontal(1))
}
