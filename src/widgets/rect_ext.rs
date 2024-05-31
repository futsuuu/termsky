pub trait RectExt {
    #[allow(dead_code)]
    fn x(self, n: impl NewValue<u16>) -> Self;
    fn y(self, n: impl NewValue<u16>) -> Self;
    #[allow(dead_code)]
    fn width(self, n: impl NewValue<u16>) -> Self;
    fn height(self, n: impl NewValue<u16>) -> Self;
}

impl RectExt for ratatui::layout::Rect {
    fn x(mut self, n: impl NewValue<u16>) -> Self {
        self.x = n.new_value(self.x);
        self
    }
    fn y(mut self, n: impl NewValue<u16>) -> Self {
        self.y = n.new_value(self.y);
        self
    }
    fn width(mut self, n: impl NewValue<u16>) -> Self {
        self.width = n.new_value(self.width);
        self
    }
    fn height(mut self, n: impl NewValue<u16>) -> Self {
        self.height = n.new_value(self.height);
        self
    }
}

pub trait NewValue<T> {
    fn new_value(self, old: T) -> T;
}

impl NewValue<u16> for u16 {
    fn new_value(self, _: u16) -> u16 {
        self
    }
}

impl<F: FnOnce(u16) -> u16> NewValue<u16> for F {
    fn new_value(self, old: u16) -> u16 {
        self(old)
    }
}
