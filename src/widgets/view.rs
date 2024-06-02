use ratatui::{buffer::Buffer, layout::Rect, widgets::WidgetRef};

use crate::{
    prelude::*,
    widgets::pages::{Home, Loading, Login},
};

#[derive(Default)]
pub struct View {
    id: ViewID,
    home: Home,
    loading: Loading,
    login: Login,
}

#[derive(Clone, Debug, Default)]
pub enum ViewID {
    Home,
    #[default]
    Loading,
    Login,
}

macro_rules! inner {
    ($self:ident) => {
        match $self.id {
            ViewID::Home => &$self.home,
            ViewID::Loading => &$self.loading,
            ViewID::Login => &$self.login,
        }
    };
    (mut $self:ident) => {
        match $self.id {
            ViewID::Home => &mut $self.home,
            ViewID::Loading => &mut $self.loading,
            ViewID::Login => &mut $self.login,
        }
    };
}

impl View {
    fn widget_ref(&self) -> &dyn WidgetRef {
        inner!(self)
    }
    fn event_handler(&self) -> &dyn crate::app::EventHandler {
        inner!(self)
    }
    fn event_handler_mut(&mut self) -> &mut dyn crate::app::EventHandler {
        inner!(mut self)
    }
}

impl WidgetRef for View {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.widget_ref().render_ref(area, buf)
    }
}

impl crate::app::EventHandler for View {
    fn on_render(&mut self, app: &mut App) {
        self.id = app.view_id().clone();
        self.event_handler_mut().on_render(app)
    }
    fn on_key(&mut self, ev: crossterm::event::KeyEvent, app: &mut App) {
        self.event_handler_mut().on_key(ev, app)
    }
    fn on_mouse(&mut self, ev: crossterm::event::MouseEvent, app: &mut App) {
        self.event_handler_mut().on_mouse(ev, app)
    }
    fn on_input(&mut self, input: tui_textarea::Input, app: &mut App) {
        self.event_handler_mut().on_input(input, app)
    }
    fn focus_in_textarea(&self) -> bool {
        self.event_handler().focus_in_textarea()
    }
}
