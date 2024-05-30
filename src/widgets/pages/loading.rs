use ratatui::{prelude::*, widgets::*};

use crate::{
    atp::Response,
    prelude::*,
    widgets::{atoms::Spinner, pages},
};

pub struct Loading {
    response: Response<crate::atp::ResumeSessionResult>,
}

impl Loading {
    pub fn new() -> Self {
        Self {
            response: Response::empty(),
        }
    }
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
            if result.is_ok() {
                app.update_view(pages::Home::new());
            } else {
                let mut login = pages::Login::new();
                login.switch_focus();
                app.update_view(login);
            }
            return;
        }

        if let TuiEvent::Key(key_event) = &ev {
            if key_event.code == crossterm::event::KeyCode::Esc {
                app.exit();
            }
        }
    }
}
