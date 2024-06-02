use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};
use tui_textarea::Key;

use crate::{
    app::App,
    atp::Response,
    widgets::{
        atoms::{Spinner, TextArea},
        ViewID,
    },
};

#[derive(Debug)]
pub struct Login {
    textareas: [TextArea<'static>; 2],
    focus: Option<usize>,
    login_res: Response<crate::atp::LoginResult>,
    resume_session_res: Response<crate::atp::ResumeSessionResult>,
}

impl Default for Login {
    fn default() -> Self {
        Self {
            textareas: [
                TextArea::new(" Handle name or Email address ", false),
                TextArea::new(" Password ", true),
            ],
            focus: None,
            login_res: Response::empty(),
            resume_session_res: Response::empty(),
        }
    }
}

impl Login {
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
        if self.login_res.is_loading() {
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
            Constraint::Min(70),
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
        if self.login_res.is_loading() || self.resume_session_res.is_loading() {
            Spinner::new().render(spinner, buf);
        }
    }
}

impl crate::app::EventHandler for Login {
    fn on_render(&mut self, app: &mut App) {
        if self.resume_session_res.is_empty() {
            self.resume_session_res = app.atp.resume_session();
        } else if let Some(result) = self.resume_session_res.take_data() {
            if result.is_ok() {
                app.set_view_id(ViewID::Home);
            } else {
                self.switch_focus();
            }
        } else if let Some(result) = self.login_res.take_data() {
            if result.is_ok() {
                app.set_view_id(ViewID::Home);
            } else {
                self.switch_focus();
            }
        }
    }

    fn on_key(&mut self, ev: crossterm::event::KeyEvent, app: &mut App) {
        if ev.code == KeyCode::Esc {
            app.exit();
            return;
        }
        if self.resume_session_res.is_loading() {
            return;
        }
        if ev.code == KeyCode::Tab {
            self.switch_focus();
        }
    }

    fn on_input(&mut self, input: tui_textarea::Input, app: &mut App) {
        if input.key == Key::Esc {
            self.lose_focus();
        } else if input.key == Key::Tab {
            self.switch_focus();
        } else if input.key == Key::Enter && self.login_res.is_empty() {
            self.login_res = app.atp.login(self.ident(), self.passwd());
            self.lose_focus();
        } else if let Some(ref mut textarea) = self.textarea() {
            textarea.input(input);
        }
    }

    fn focus_in_textarea(&self) -> bool {
        self.has_focus()
    }
}
