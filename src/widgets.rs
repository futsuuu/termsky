pub mod atoms;
pub mod molecules;
pub mod organisms;
pub mod pages;
mod post;
mod store;
pub mod templates;
mod view;

pub use post::{Posts, PostsState};
use store::{Store, Storeable};
pub use view::View;

pub trait RectExt {
    fn x(self, n: u16) -> Self;
    fn y(self, n: u16) -> Self;
    fn width(self, n: u16) -> Self;
    fn height(self, n: u16) -> Self;
}

impl RectExt for ratatui::layout::Rect {
    fn x(mut self, n: u16) -> Self {
        self.x = n;
        self
    }
    fn y(mut self, n: u16) -> Self {
        self.y = n;
        self
    }
    fn width(mut self, n: u16) -> Self {
        self.width = n;
        self
    }
    fn height(mut self, n: u16) -> Self {
        self.height = n;
        self
    }
}
