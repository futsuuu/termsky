use ratatui::{prelude::*, widgets::*};

use crate::{
    prelude::*,
    widgets::{atoms::Spinner, pages},
};

pub struct Loading;

impl Loading {
    pub fn new() -> Self {
        Self
    }
}

impl WidgetRef for Loading {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Spinner::new().render_ref(area, buf);
    }
}

impl AppHandler for Loading {
    fn atp_response(&mut self, app: &mut App, res: AtpResponse) {
        if let AtpResponse::Session(session) = res {
            if session.is_some() {
                app.update_view(pages::Home::new());
            } else {
                let mut login = pages::Login::new();
                login.unblock_input();
                app.update_view(login);
            }
        }
    }

    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {
        if let TuiEvent::Key(key_event) = &ev {
            if key_event.code == crossterm::event::KeyCode::Esc {
                app.exit();
            }
        }
    }
}
