use ratatui::{prelude::*, widgets::*};

use crate::{
    atp::Response,
    prelude::*,
    widgets::{
        atoms::{Spinner, TextArea},
        pages,
    },
};

#[derive(Debug)]
pub struct Login {
    textareas: [TextArea<'static>; 2],
    focus: Option<usize>,
    response: Response<crate::atp::LoginResult>,
}

impl Default for Login {
    fn default() -> Self {
        Self::new()
    }
}

impl Login {
    pub fn new() -> Self {
        Self {
            textareas: [
                TextArea::new(" Handle name or Email address ", false),
                TextArea::new(" Password ", true),
            ],
            focus: None,
            response: Response::empty(),
        }
    }

    pub fn ident(&self) -> String {
        self.textareas[0].lines()[0].to_string()
    }
    pub fn passwd(&self) -> String {
        self.textareas[1].lines()[0].to_string()
    }

    pub fn textarea(&mut self) -> Option<&mut TextArea<'static>> {
        self.focus.map(|n| &mut self.textareas[n])
    }

    pub fn switch_focus(&mut self) {
        if self.response.is_loading() {
            return;
        }
        if let Some(n) = self.focus {
            self.textareas[n].lose_focus();
        }
        self.focus = match self.focus {
            Some(1) => Some(0),
            Some(0) => Some(1),
            None => Some(0),
            _ => unreachable!(),
        };
        if let Some(n) = self.focus {
            self.textareas[n].set_focus();
        }
    }

    pub fn lose_focus(&mut self) {
        if let Some(n) = self.focus {
            self.textareas[n].lose_focus();
        }
        self.focus = None;
    }

    pub fn has_focus(&self) -> bool {
        self.focus.is_some()
    }
}

impl WidgetRef for Login {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [_, area, _] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Min(50),
            Constraint::Percentage(30),
        ])
        .areas(area);
        let [_, ident, passwd, spinner, _] = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(area);

        self.textareas[0].widget().render(ident, buf);
        self.textareas[1].widget().render(passwd, buf);
        if self.response.is_loading() {
            Spinner::new().render(spinner, buf);
        }
    }
}

impl AppHandler for Login {
    fn tui_event(&mut self, app: &mut App, ev: TuiEvent) {
        if let Some(result) = self.response.take_data() {
            if result.is_ok() {
                app.update_view(pages::Home::new());
            } else {
                self.switch_focus();
            }
            return;
        }

        if let TuiEvent::Key(ev) = ev {
            if ev.code == KeyCode::Esc {
                if self.has_focus() {
                    self.lose_focus();
                } else {
                    app.exit();
                }
                return;
            } else if ev.code == KeyCode::Tab {
                self.switch_focus();
                return;
            } else if ev.code == KeyCode::Enter && self.response.is_empty() {
                self.response = app.atp.login(self.ident(), self.passwd());
                self.lose_focus();
                return;
            }
        }

        if let Some(ref mut textarea) = self.textarea() {
            textarea.input(ev);
        }
    }
}
