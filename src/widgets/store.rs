use ratatui::{prelude::*, widgets::*};

pub struct Store<'a> {
    widgets: Vec<(Rect, Box<dyn WidgetRef + 'a>)>,
    pub scroll_v: i16,
    // Currently not needed
    // pub scroll_h: i16,
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
            scroll_v: 0,
        }
    }

    pub fn stored_area(&self) -> Rect {
        self.widgets
            .iter()
            .map(|(area, _widget)| *area)
            .reduce(|acc, area| acc.union(area))
            .unwrap_or_default()
    }

    pub fn scroll_v(mut self, n: i16) -> Self {
        self.scroll_v = n;
        self
    }
}

impl WidgetRef for Store<'_> {
    // ```text
    // ╔ buf ══════════════════════════════════════════════╗
    // ║                                                   ║
    // ║    ┌ content ────────────────────────────────┐    ║
    // ║    │                                         │    ║
    // ║    │        ╔═════════════╗    ─┰─           │    ║
    // ║    │        ║             ║     ┃            │    ║
    // ║    │        ║             ║     ┃ scroll_v   │    ║
    // ║    │        ║ viewport    ║     ┃            │    ║
    // ║    │        ╚═════════════╝     ┃            │    ║
    // ║    │                            ▼            │    ║
    // ║    │               ┌──────────────┐          │    ║
    // ║    │        ┝━━━━━►│              │          │    ║
    // ║    │      scroll_h │              │          │    ║
    // ║    │               │ render_area  │          │    ║
    // ║    │               └──────────────┘          │    ║
    // ║    │                                         │    ║
    // ║    │ content_area                            │    ║
    // ║    └─────────────────────────────────────────┘    ║
    // ║                                                   ║
    // ╚═══════════════════════════════════════════════════╝
    // ```
    fn render_ref(&self, viewport: Rect, buf: &mut Buffer) {
        let content_area = self.stored_area();
        let render_area = {
            let y = viewport.y.saturating_add_signed(self.scroll_v);
            // `height` is not always equal to `viewport.height` if `scroll_v` < 0
            let height = (viewport.bottom() as i16 + self.scroll_v) as u16 - y;
            content_area.intersection(Rect {
                y,
                height,
                ..viewport
            })
        };
        let content = {
            let mut buf = Buffer::empty(content_area);
            for (widget_area, widget) in &self.widgets {
                if render_area.intersects(*widget_area) {
                    widget.render_ref(*widget_area, &mut buf);
                }
            }
            buf
        };
        for x in render_area.left()..render_area.right() {
            for y in render_area.top()..render_area.bottom() {
                let ry = y.checked_add_signed(-self.scroll_v).unwrap();
                *buf.get_mut(x, ry) = content.get(x, y).clone();
            }
        }
    }
}

impl<'a, W: WidgetRef + 'a> Storeable<'a> for W {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        store.widgets.push((area, Box::new(self)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::assert_buffer_eq;

    #[test]
    fn stored_area() {
        let mut store = Store::new();

        Block::new().store(Rect::new(0, 0, 1, 1), &mut store);
        assert_eq!(Rect::new(0, 0, 1, 1), store.stored_area());

        Block::new().store(Rect::new(1, 1, 1, 1), &mut store);
        assert_eq!(Rect::new(0, 0, 2, 2), store.stored_area());

        Block::new().store(Rect::new(3, 0, 1, 1), &mut store);
        assert_eq!(Rect::new(0, 0, 4, 2), store.stored_area());

        Block::new().store(Rect::new(0, 0, 5, 5), &mut store);
        assert_eq!(Rect::new(0, 0, 5, 5), store.stored_area());
    }

    #[test]
    fn render_widgets() {
        let mut store = Store::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));

        Span::from("hello").store(Rect::new(0, 0, 5, 1), &mut store);
        Span::from("world").store(Rect::new(7, 4, 5, 1), &mut store);
        store.render(buf.area, &mut buf);
        assert_buffer_eq!(
            buf,
            Buffer::with_lines(vec![
                "hello     ",
                "          ",
                "          ",
                "          ",
                "       wor",
            ])
        );
    }

    #[test]
    fn render_small_widget() {
        let mut store = Store::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));

        Span::from("#").store(Rect::new(4, 2, 2, 1), &mut store);
        store.render(buf.area, &mut buf);
        assert_buffer_eq!(
            buf,
            Buffer::with_lines(vec![
                "          ",
                "          ",
                "    #     ",
                "          ",
                "          ",
            ])
        );
    }

    #[test]
    fn scroll_vertical_positive() {
        let mut store = Store::new().scroll_v(2);
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));

        Span::from("11").store(Rect::new(4, 1, 2, 1), &mut store);
        Span::from("22").store(Rect::new(4, 2, 2, 1), &mut store);
        store.render(buf.area, &mut buf);
        assert_buffer_eq!(
            buf,
            Buffer::with_lines(vec![
                "    22    ",
                "          ",
                "          ",
                "          ",
                "          ",
            ])
        );
    }

    #[test]
    fn scroll_vertical_negative() {
        let mut store = Store::new().scroll_v(-2);
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));

        Span::from("11").store(Rect::new(4, 2, 2, 1), &mut store);
        Span::from("22").store(Rect::new(4, 3, 2, 1), &mut store);
        store.render(buf.area, &mut buf);
        assert_buffer_eq!(
            buf,
            Buffer::with_lines(vec![
                "          ",
                "          ",
                "          ",
                "          ",
                "    11    ",
            ])
        );
    }
}
