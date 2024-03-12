use ratatui::{prelude::*, widgets::*};

#[derive(Clone, Debug)]
pub struct Home {}

impl Home {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &Home {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Home")
            .block(Block::default().title("home"))
            .render(area, buf);
    }
}
