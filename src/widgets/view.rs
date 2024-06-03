use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::WidgetRef,
};

use crate::{
    prelude::*,
    widgets::{
        molecules::Tab,
        organisms::TabBar,
        pages::{Home, Login},
    },
};

#[derive(Default)]
pub struct View {
    id: ViewID,
    home: Home,
    login: Login,
}

macro_rules! inner {
    ($self:ident) => {
        match $self.id {
            ViewID::Home => &$self.home,
            ViewID::Login { .. } => &$self.login,
        }
    };
    (mut $self:ident) => {
        match $self.id {
            ViewID::Home => &mut $self.home,
            ViewID::Login { .. } => &mut $self.login,
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
        let [tabbar_area, main_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(4)])
                .horizontal_margin(1)
                .areas(area);

        TabBar::from_iter([
            Tab::new("1. Login").selected(matches!(self.id, ViewID::Login { .. })),
            Tab::new("2. Home").selected(matches!(self.id, ViewID::Home)),
            Tab::new("3. Settings").active(false),
        ])
        .render_ref(tabbar_area, buf);

        self.widget_ref().render_ref(main_area, buf)
    }
}

impl crate::app::EventHandler for View {
    fn on_render(&mut self, app: &mut App) {
        self.id = app.view_id().clone();
        self.event_handler_mut().on_render(app)
    }

    fn on_key(&mut self, ev: crossterm::event::KeyEvent, app: &mut App) {
        if ev.code == KeyCode::Char('1') {
            app.set_view_id(ViewID::Login {
                resume_session: false,
            });
        } else if ev.code == KeyCode::Char('2') {
            app.set_view_id(ViewID::Home);
        }
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

#[derive(Clone, Debug, PartialEq)]
pub enum ViewID {
    Login { resume_session: bool },
    Home,
}

impl Default for ViewID {
    fn default() -> Self {
        Self::Login {
            resume_session: true,
        }
    }
}

impl ViewID {
    pub fn login_resume_session(&self) -> bool {
        if let Self::Login { resume_session, .. } = self {
            *resume_session
        } else {
            false
        }
    }
}
