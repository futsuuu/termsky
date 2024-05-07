use ratatui::{prelude::*, widgets::*};
#[cfg(test)]
use rstest::*;

use crate::prelude::*;

pub struct Store<'a> {
    widgets: Vec<(Rect, Box<dyn WidgetRef + 'a>)>,
    stored_area: Option<Rect>,
    pub scroll_v: i32,
    // Currently not needed
    // pub scroll_h: i32,
}

pub trait Storeable<'a> {
    #[allow(unused_variables)]
    fn store(self, area: Rect, store: &mut Store<'a>);
}

impl Store<'_> {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            stored_area: None,
            scroll_v: 0,
        }
    }

    #[inline]
    pub fn stored_area(&self) -> Rect {
        self.stored_area.unwrap_or_default()
    }

    pub fn scroll_v(mut self, n: i32) -> Self {
        self.scroll_v = n;
        self
    }

    pub fn extend(&mut self, other: Self) {
        self.widgets.extend(other.widgets);
        self.stored_area = match (self.stored_area, other.stored_area) {
            (Some(r1), Some(r2)) => Some(r1.union(r2)),
            (Some(r), None) | (None, Some(r)) => Some(r),
            (None, None) => None,
        };
    }

    pub fn bottom_space(&self, area: Rect) -> Rect {
        match self.stored_area {
            Some(stored_area) => bottom_space(stored_area, area),
            None => area,
        }
    }
}

impl WidgetRef for Store<'_> {
    // ```text
    // ╔ buf ═════════════════════════════════════════════════╗
    // ║                                                      ║
    // ║    ┌ content ───────────────────────────────────┐    ║
    // ║    │                                            │    ║
    // ║    │        ╔════════════════╗    ─┰─           │    ║
    // ║    │        ║                ║     ┃            │    ║
    // ║    │        ║                ║     ┃ scroll_v   │    ║
    // ║    │        ║ viewport       ║     ┃            │    ║
    // ║    │        ╚════════════════╝     ┃            │    ║
    // ║    │                               ▼            │    ║
    // ║    │               ┌─────────────────┐          │    ║
    // ║    │        ┝━━━━━►│                 │          │    ║
    // ║    │      scroll_h │                 │          │    ║
    // ║    │               │ rendering_area  │          │    ║
    // ║    │               └─────────────────┘          │    ║
    // ║    │                                            │    ║
    // ║    │ stored_area                                │    ║
    // ║    └────────────────────────────────────────────┘    ║
    // ║                                                      ║
    // ╚══════════════════════════════════════════════════════╝
    // ```
    fn render_ref(&self, viewport: Rect, buf: &mut Buffer) {
        let stored_area = self.stored_area();
        let rendering_area = scroll_rect(viewport, self.scroll_v).intersection(stored_area);
        let content = {
            let content_size = (stored_area.width as usize)
                .checked_mul(stored_area.height as usize)
                .expect("stored widgets are too large");
            let mut buf = Buffer {
                area: stored_area,
                content: vec![ratatui::buffer::Cell::default(); content_size],
            };
            for (widget_area, widget) in &self.widgets {
                if rendering_area.intersects(*widget_area) {
                    widget.render_ref(*widget_area, &mut buf);
                }
            }
            buf
        };
        for x in rendering_area.left()..rendering_area.right() {
            for y in rendering_area.top()..rendering_area.bottom() {
                // `rendering_area` already has `scroll_v` added
                let ry = (y as i32 - self.scroll_v) as u16;
                *buf.get_mut(x, ry) = content.get(x, y).clone();
            }
        }
    }
}

impl<'a, W: WidgetRef + 'a> Storeable<'a> for W {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        store.widgets.push((area, Box::new(self)));
        store.stored_area = Some(match store.stored_area {
            Some(stored_area) => stored_area.union(area),
            None => area,
        });
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

fn bottom_space(stored_area: Rect, viewport: Rect) -> Rect {
    let stored_area = stored_area.intersection(viewport);
    let height = viewport.bottom().saturating_sub(stored_area.bottom());
    viewport.y(viewport.bottom() - height).height(height)
}

#[cfg(test)]
#[rstest]
#[case(Rect::new(0, 7, 10, 3), Rect::new(3, 5, 1, 2), Rect::new(0, 0, 10, 10))]
#[case::empty_store(
    Rect::new(0, 0, 10, 10),
    Rect::new(0, 0, 0, 0),
    Rect::new(0, 0, 10, 10)
)]
#[case::no_space(
    Rect::new(0, 10, 10, 0),
    Rect::new(0, 0, 10, 10),
    Rect::new(0, 0, 10, 10)
)]
fn test_bottom_space(#[case] space: Rect, #[case] stored: Rect, #[case] rendering: Rect) {
    assert_eq!(space, bottom_space(stored, rendering));
}

fn scroll_rect(area: Rect, scroll_v: i32) -> Rect {
    let y = i32::from(area.y) + scroll_v;
    let height = if y.is_negative() {
        (i32::from(area.height) + y).clamp(0, i32::from(u16::MAX)) as u16
    } else {
        area.height
    };
    area.y(y.clamp(0, i32::from(u16::MAX)) as u16)
        .height(height)
}

#[cfg(test)]
#[rstest]
#[case::no_scroll(Rect::new(0, 0, 10, 10), Rect::new(0, 0, 10, 10), 0)]
#[case::scroll(Rect::new(0, 5, 10, 10), Rect::new(0, 0, 10, 10), 5)]
#[case::resize(
    Rect::new(0, 0, 10, 3),
    Rect::new(0, 0, 10, 10),
    -7
)]
#[case::out_of_buf(
    Rect::new(0, 0, 10, 0),
    Rect::new(0, 0, 10, 10),
    -i32::MAX
)]
fn test_scroll_rect(#[case] scrolled: Rect, #[case] area: Rect, #[case] scroll_v: i32) {
    assert_eq!(scrolled, scroll_rect(area, scroll_v));
}
