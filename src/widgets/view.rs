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

impl WidgetRef for View {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        match self.id {
            ViewID::Home => self.home.render_ref(area, buf),
            ViewID::Loading => self.loading.render_ref(area, buf),
            ViewID::Login => self.login.render_ref(area, buf),
        }
    }
}

impl AppHandler for View {
    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {
        self.id = app.view_id().clone();
        match self.id {
            ViewID::Home => self.home.tui_event(app, ev),
            ViewID::Loading => self.loading.tui_event(app, ev),
            ViewID::Login => self.login.tui_event(app, ev),
        }
    }
}
