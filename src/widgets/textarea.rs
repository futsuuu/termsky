use std::{fmt, ops};

use ratatui::{prelude::*, widgets::*};
use tui_textarea::TextArea;

#[derive(Clone)]
pub struct Wrapper<'a> {
    title: &'a str,
    inner: TextArea<'a>,
}

impl<'a> Wrapper<'a> {
    pub fn new(title: &'a str, mask: bool) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::new().not_underlined());
        textarea.set_selection_style(Style::new().reversed());
        if mask {
            textarea.set_mask_char('·');
        }
        let mut t = Self {
            title,
            inner: textarea,
        };
        t.lose_focus();
        t
    }

    pub fn set_focus(&mut self) {
        self.inner.set_cursor_style(Style::new().reversed());
        self.inner.set_block(block(self.title).blue().bold());
    }

    pub fn lose_focus(&mut self) {
        self.inner.set_cursor_style(Style::new().hidden());
        self.inner.set_block(block(self.title).dim());
    }

    pub fn widget(&'a self) -> impl Widget + 'a {
        self.inner.widget()
    }
}

fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .title(title)
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
}

impl fmt::Debug for Wrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.mask_char().is_some() {
            f.write_str("***")
        } else {
            f.write_str(self.inner.lines().join("\n").as_str())
        }
    }
}

impl<'a> ops::Deref for Wrapper<'a> {
    type Target = TextArea<'a>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> ops::DerefMut for Wrapper<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
