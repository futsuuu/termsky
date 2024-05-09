use ratatui::{prelude::*, widgets::*};

use crate::{
    prelude::*,
    widgets::{molecules::Tab, Store, Storeable},
};

pub struct TabBar {
    tabs: Vec<Tab>,
}

impl<T: Into<Tab>> FromIterator<T> for TabBar {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            tabs: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl WidgetRef for TabBar {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut store = Store::new();
        let sep = ratatui::symbols::line::HORIZONTAL
            .repeat(usize::from(area.width))
            .blue()
            .dim();
        for (i, tab) in self.tabs.clone().into_iter().enumerate() {
            if i != 0 {
                Text::from(sep.clone()).store(store.bottom_space(area).height(1), &mut store);
            }
            tab.store(store.bottom_space(area), &mut store);
        }
        store.render_ref(area, buf);
    }
}
