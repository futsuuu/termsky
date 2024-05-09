use ratatui::{prelude::*, widgets::*};

use crate::widgets::{atoms::Text, Store, Storeable};

#[derive(Clone)]
pub struct Tab {
    text: String,
    active: bool,
    selected: bool,
}

impl Tab {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            active: true,
            selected: false,
        }
    }

    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }
}

impl Storeable<'_> for Tab {
    fn store(self, area: Rect, store: &mut Store) {
        let style = if !self.active {
            Style::new().dim()
        } else if self.selected {
            Style::new().bold()
        } else {
            Style::new()
        };
        Text::from_iter([
            if self.selected {
                ratatui::symbols::line::THICK_VERTICAL
            } else {
                " "
            }
            .blue(),
            " ".into(),
            self.text.clone().set_style(style),
        ])
        .block(Block::new().padding(Padding::vertical(1)))
        .ignore_if_empty(false)
        .store(area, store);
    }
}
