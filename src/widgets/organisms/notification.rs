use ratatui::{layout::Rect, style::Stylize};

use crate::widgets::{atoms::Text, Store, Storeable};

#[derive(Clone)]
pub struct Notification {
    title: Option<Text>,
    is_read: bool,
}

impl Notification {
    pub fn is_read(&self) -> bool {
        self.is_read
    }
}

impl From<crate::atp::Notification> for Notification {
    fn from(value: crate::atp::Notification) -> Self {
        let author = crate::atp::types::Account::from(&value.author).name.bold();
        Self {
            title: match value.reason.as_str() {
                "like" => Some(Text::from_iter([author, " liked your post".into()])),
                "repost" => Some(Text::from_iter([author, " reposted your post".into()])),
                "follow" => Some(Text::from_iter([author, " followed you".into()])),
                "mention" => None,
                "reply" => None,
                "quote" => None,
                _ => None,
            },
            is_read: value.is_read,
        }
    }
}

impl Storeable<'_> for Notification {
    fn store(self, area: Rect, store: &mut Store<'_>) {
        if let Some(title) = self.title.clone() {
            title.store(area, store);
        }
    }
}
