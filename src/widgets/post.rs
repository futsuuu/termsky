use atrium_api::{
    app::bsky::{self, feed::defs::FeedViewPost},
    records,
    types::Union,
};
use ratatui::{prelude::*, widgets::*};

use crate::{
    prelude::*,
    widgets::{
        atoms::{BlockExt, Text},
        Store, Storeable,
    },
};

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
            Block::new()
                .borders(Borders::BOTTOM)
                .border_style(Style::new().blue().dim())
                .wrap_child(post)
                .fit_vertical()
                .store(store.bottom_space(area.height(u16::MAX)), &mut store);
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
        content: Text,
        likes: u64,
        replies: u64,
        reposts: u64,
        reposted_by: Option<Account>,
        embed: Option<enum Embed {
            Media(enum EmbedMedia {
                External(struct EmbedExternal {
                    title: Text,
                    description: Text,
                    uri: Text,
                }),
                Image(Vec<struct EmbedImage {
                    alt: Text,
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
                    record.text.clone().into()
                }
                _ => "unimplemented!".into(),
            },
            likes: post.like_count.unwrap_or(0) as u64,
            replies: post.reply_count.unwrap_or(0) as u64,
            reposts: post.repost_count.unwrap_or(0) as u64,
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

impl From<Box<bsky::embed::record::ViewRecord>> for Post {
    fn from(value: Box<bsky::embed::record::ViewRecord>) -> Self {
        Self {
            author: value.author.into(),
            content: match &value.value {
                records::Record::Known(records::KnownRecord::AppBskyFeedPost(record)) => {
                    record.text.clone().into()
                }
                _ => "unimplemented!".into(),
            },
            likes: value.like_count.unwrap_or(0) as u64,
            replies: value.reply_count.unwrap_or(0) as u64,
            reposts: value.repost_count.unwrap_or(0) as u64,
            reposted_by: None,
            embed: value
                .embeds
                .and_then(|e| {
                    e.into_iter().find_map(|e| match e {
                        Union::Refs(e) => Some(e),
                        _ => None,
                    })
                })
                .map(Into::into),
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
            ViewRecord(record) => Self::Post(Box::new(Post::from(record))),
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
        Self::External(value.external.into())
    }
}

impl From<Box<bsky::embed::images::View>> for EmbedMedia {
    fn from(value: Box<bsky::embed::images::View>) -> Self {
        Self::Image(value.images.into_iter().map(Into::into).collect())
    }
}

impl From<bsky::embed::external::ViewExternal> for EmbedExternal {
    fn from(value: bsky::embed::external::ViewExternal) -> Self {
        Self {
            title: Text::from(value.title.bold()).alignment(Alignment::Center),
            description: Text::from(value.description),
            uri: Text::from(value.uri.dim()).ignore_if_empty(false),
        }
    }
}

impl From<bsky::embed::images::ViewImage> for EmbedImage {
    fn from(value: bsky::embed::images::ViewImage) -> Self {
        Self {
            alt: Text::from_iter(["  ".magenta(), value.alt.into()]),
        }
    }
}

impl<'a> Storeable<'a> for &'a Post {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        if let Some(reposted_by) = &self.reposted_by {
            Text::from(format!("  Reposted by {}", reposted_by.name))
                .store(store.bottom_space(area).height(1), store);
        }
        Block::new()
            .padding(Padding::bottom(1))
            .wrap_child(Text::from_iter({
                let mut spans = vec![self.author.name.clone().bold()];
                if let Some(opt_name) = &self.author.opt_name {
                    spans.extend(["  ".into(), opt_name.clone().dim().italic()]);
                }
                spans
            }))
            .fit_vertical()
            .store(store.bottom_space(area), store);
        self.content.clone().store(store.bottom_space(area), store);
        if let Some(embed) = &self.embed {
            embed.store(store.bottom_space(area), store);
        }
        Block::new()
            .padding(Padding::top(1))
            .wrap_child(Text::from(format!(
                " {}    {}   ♥ {}",
                self.replies, self.reposts, self.likes
            )))
            .fit_vertical()
            .store(store.bottom_space(area), store);
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
                record.store(store.bottom_space(area), store);
            }
            Embed::Unimplemented => {
                embed_block()
                    .wrap_child(Text::from("unimplemented!"))
                    .fit_vertical()
                    .store(area, store);
            }
        }
    }
}

impl<'a> Storeable<'a> for &'a EmbedRecord {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        let block = embed_block();
        match self {
            EmbedRecord::Post(post) => block.wrap_child(post.as_ref()),
            EmbedRecord::NotFound => block.wrap_child(Text::from("  Not Found")),
            EmbedRecord::Blocked => block.wrap_child(Text::from("  Blocked")),
            EmbedRecord::Unimplemented => block.wrap_child(Text::from("unimplemented!")),
        }
        .fit_vertical()
        .store(area, store);
    }
}

impl<'a> Storeable<'a> for &'a EmbedMedia {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        match self {
            EmbedMedia::External(external) => {
                embed_block()
                    .wrap(|inner, s| {
                        Block::new()
                            .borders(Borders::BOTTOM)
                            .border_set(ratatui::symbols::border::ONE_EIGHTH_WIDE)
                            .wrap_child(external.title.clone())
                            .fit_all()
                            .store(s.bottom_space(inner).height(3), s);
                        Block::new()
                            .padding(Padding::bottom(1))
                            .wrap_child(external.description.clone())
                            .fit_vertical()
                            .store(s.bottom_space(inner).height(3), s);
                        external.uri.clone().store(s.bottom_space(inner).height(1), s);
                    })
                    .fit_vertical()
                    .store(area, store);
            }
            EmbedMedia::Image(images) => {
                for image in images {
                    embed_block()
                        .wrap_child(image.alt.clone())
                        .fit_vertical()
                        .store(store.bottom_space(area), store);
                }
            }
        }
    }
}

fn embed_block() -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dim())
        .padding(Padding::horizontal(1))
}
