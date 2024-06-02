use ratatui::{prelude::*, widgets::*};

use crate::{
    atp::Response,
    prelude::*,
    widgets::{atoms::Spinner, ViewID},
};

#[derive(Default)]
pub struct Loading {
    response: Response<crate::atp::ResumeSessionResult>,
}

impl WidgetRef for Loading {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Spinner::new().render_ref(area, buf);
    }
}

impl crate::app::EventHandler for Loading {
    fn on_render(&mut self, app: &mut App) {
        if self.response.is_empty() {
            self.response = app.atp.resume_session();
        }

        if let Some(result) = self.response.take_data() {
            app.set_view_id(if result.is_ok() {
                ViewID::Home
            } else {
                ViewID::Login
            });
        }
    }

    fn on_key(&mut self, ev: crossterm::event::KeyEvent, app: &mut App) {
        if ev.code == crossterm::event::KeyCode::Esc {
            app.exit();
        }
    }
}
