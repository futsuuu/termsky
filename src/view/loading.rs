use ratatui::{prelude::*, widgets::*};

use crate::widgets::Spinner;

pub struct Loading;

impl Loading {
    pub fn new() -> Self {
        Self
    }
}

impl WidgetRef for Loading {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Spinner::new().render_ref(area, buf);
    }
}
