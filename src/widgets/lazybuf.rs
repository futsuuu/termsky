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
            .fold(Rect::default(), |r, (area, _)| r.union(*area))
    }
}

impl WidgetRef for LazyBuffer<'_> {
    fn render_ref(&self, _area: Rect, buf: &mut Buffer) {
        let mut vbuf = Buffer::empty(self.rendered_area());
        for (widget_area, widget) in &self.widgets {
            widget.render_ref(*widget_area, &mut vbuf);
        }
        buf.merge(&vbuf);
    }
}

impl<'a, W: WidgetRef + 'a> LazyWidget<'a> for W {
    fn render_lazy(self, area: Rect, buf: &mut LazyBuffer<'a>) {
        buf.widgets.push((area, Box::new(self)));
    }
}
