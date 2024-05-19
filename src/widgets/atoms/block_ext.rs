use ratatui::{prelude::*, widgets::*};
#[cfg(test)]
use rstest::rstest;

use crate::widgets::{Store, Storeable};

pub trait BlockExt<'a>: Sized {
    fn wrap(self, func: impl FnOnce(Rect, &mut Store<'a>) + 'a) -> Wrapper<'a>;

    fn wrap_child(self, storeable: impl Storeable<'a> + 'a) -> Wrapper<'a> {
        self.wrap(|area, store| {
            storeable.store(area, store);
        })
    }
}

impl<'a> BlockExt<'a> for Block<'a> {
    fn wrap(self, func: impl FnOnce(Rect, &mut Store<'a>) + 'a) -> Wrapper<'a> {
        Wrapper {
            block: self,
            func: Box::new(func),
            fit: Fit::default(),
        }
    }
}

pub struct Wrapper<'a> {
    block: Block<'a>,
    #[allow(clippy::type_complexity)]
    func: Box<dyn FnOnce(Rect, &mut Store<'a>) + 'a>,
    fit: Fit,
}

impl<'a> Wrapper<'a> {
    pub fn fit_all(self) -> Self {
        self.fit_vertical().fit_horizontal()
    }
    pub fn fit_vertical(self) -> Self {
        self.fit_top().fit_bottom()
    }
    pub fn fit_horizontal(self) -> Self {
        self.fit_right().fit_left()
    }
    pub fn fit_top(mut self) -> Self {
        self.fit.top = true;
        self
    }
    pub fn fit_bottom(mut self) -> Self {
        self.fit.bottom = true;
        self
    }
    pub fn fit_right(mut self) -> Self {
        self.fit.right = true;
        self
    }
    pub fn fit_left(mut self) -> Self {
        self.fit.left = true;
        self
    }
}

impl<'a> Storeable<'a> for Wrapper<'a> {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        let inner = self.block.inner(area);
        let mut s = Store::new();
        (self.func)(inner, &mut s);
        let stored_area = s.stored_area();
        if stored_area.is_empty() {
            return;
        }
        store.extend(s);
        self.block
            .store(self.fit.calc(area, inner, stored_area), store);
    }
}

#[derive(Default)]
struct Fit {
    top: bool,
    bottom: bool,
    right: bool,
    left: bool,
}

impl Fit {
    fn calc(&self, area: Rect, inner: Rect, target: Rect) -> Rect {
        let left = if self.left {
            target.left().saturating_sub(inner.left() - area.left())
        } else {
            area.left()
        };
        let top = if self.top {
            target.top().saturating_sub(inner.top() - area.top())
        } else {
            area.top()
        };
        let right = if self.right {
            target.right().saturating_add(area.right() - inner.right())
        } else {
            area.right()
        };
        let bottom = if self.bottom {
            target.bottom().saturating_add(area.bottom() - inner.bottom())
        } else {
            area.bottom()
        };
        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    #[cfg(test)]
    fn vertical() -> Self {
        Self {
            top: true,
            bottom: true,
            ..Default::default()
        }
    }
}

#[cfg(test)]
#[rstest]
#[case(
    Rect::new(0, 2, 10, 6),
    Fit::vertical(),
    Rect::new(0, 0, 10, 10),
    Rect::new(1, 1, 8, 8),
    Rect::new(3, 3, 4, 4)
)]
#[case::max_height(
    Rect::new(0, 0, 10, 3),
    Fit::vertical(),
    Rect {
        x: 0,
        y: 0,
        width: 10,
        height: u16::MAX,
    },
    Rect {
        x: 1,
        y: 1,
        width: 8,
        height: u16::MAX - 2,
    },
    Rect::new(1, 1, 8, 1)
)]
fn test_fit(
    #[case] result: Rect,
    #[case] fit: Fit,
    #[case] area: Rect,
    #[case] inner: Rect,
    #[case] target: Rect,
) {
    assert_eq!(result, fit.calc(area, inner, target));
}
