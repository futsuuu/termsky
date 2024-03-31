use ratatui::{prelude::*, widgets::*};

pub struct Store<'a> {
    widgets: Vec<(Rect, Box<dyn WidgetRef + 'a>)>,
}

pub trait Storeable<'a> {
    #[allow(unused_variables)]
    fn store(self, area: Rect, store: &mut Store<'a>)
    where
        Self: 'a;
}

impl Store<'_> {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub fn stored_area(&self) -> Rect {
        self.widgets
            .iter()
            .map(|(area, _widget)| *area)
            .reduce(|acc, area| acc.union(area))
            .unwrap_or_default()
    }
}

impl WidgetRef for Store<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let stored_area = self.stored_area();
        let mut vbuf = Buffer::empty(stored_area);
        for (widget_area, widget) in &self.widgets {
            widget.render_ref(*widget_area, &mut vbuf);
        }
        let area = area.intersection(buf.area).intersection(stored_area);
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                *buf.get_mut(x, y) = vbuf.get(x, y).clone();
            }
        }
    }
}

impl<'a, W: WidgetRef + 'a> Storeable<'a> for W {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        store.widgets.push((area, Box::new(self)));
    }
}
