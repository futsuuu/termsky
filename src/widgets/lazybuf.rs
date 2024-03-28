use ratatui::{prelude::*, widgets::*};

pub struct LazyBuffer<'a> {
    widgets: Vec<(Rect, Box<dyn WidgetRef + 'a>)>,
}

pub trait LazyWidget<'a> {
    #[allow(unused_variables)]
    fn render_lazy(self, area: Rect, buf: &mut LazyBuffer<'a>)
    where
        Self: 'a;
}

impl LazyBuffer<'_> {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub fn rendered_area(&self) -> Rect {
        self.widgets
            .iter()
            .map(|(area, _widget)| *area)
            .reduce(|acc, area| acc.union(area))
            .unwrap_or_default()
    }
}

impl WidgetRef for LazyBuffer<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let rendered_area = self.rendered_area();
        let mut vbuf = Buffer::empty(rendered_area);
        for (widget_area, widget) in &self.widgets {
            widget.render_ref(*widget_area, &mut vbuf);
        }
        let area = area.intersection(buf.area).intersection(rendered_area);
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                *buf.get_mut(x, y) = vbuf.get(x, y).clone();
            }
        }
    }
}

impl<'a, W: WidgetRef + 'a> LazyWidget<'a> for W {
    fn render_lazy(self, area: Rect, buf: &mut LazyBuffer<'a>) {
        buf.widgets.push((area, Box::new(self)));
    }
}
