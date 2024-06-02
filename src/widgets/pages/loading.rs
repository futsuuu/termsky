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

impl AppHandler for Loading {
    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {
        if self.response.is_empty() {
            self.response = app.atp.resume_session();
        }

        if let Some(result) = self.response.take_data() {
            app.set_view_id(if result.is_ok() {
                ViewID::Home
            } else {
                ViewID::Login
            });
            return;
        }

        if let TuiEvent::Key(key_event) = &ev {
            if key_event.code == crossterm::event::KeyCode::Esc {
                app.exit();
            }
        }
    }
}
