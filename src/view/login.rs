use std::fmt;

use ratatui::{prelude::*, widgets::*};
use tui_textarea::TextArea;

#[derive(Clone)]
pub struct Login {
    focus: Focus,
    error: Option<String>,
    wait: bool,
    ident: TextArea<'static>,
    passwd: TextArea<'static>,
}

impl fmt::Debug for Login {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Login")
            .field("focus", &self.focus)
            .field("error", &self.error)
            .field("wait", &self.wait)
            .field("ident", &self.ident.lines()[0])
            .field("passwd", &"***")
            .finish()
    }
}

#[derive(Clone, Debug)]
enum Focus {
    Ident,
    Passwd,
    None,
}

impl Login {
    pub fn new() -> Self {
        Self {
            focus: Focus::Ident,
            error: None,
            wait: false,
            ident: create_textarea(false),
            passwd: create_textarea(true),
        }
    }

    pub fn get_ident(&self) -> String {
        self.ident.lines()[0].to_string()
    }

    pub fn get_passwd(&self) -> String {
        self.passwd.lines()[0].to_string()
    }

    pub fn textarea(&mut self) -> Option<&mut TextArea<'static>> {
        match self.focus {
            Focus::Ident => Some(&mut self.ident),
            Focus::Passwd => Some(&mut self.passwd),
            Focus::None => None,
        }
    }

    pub fn switch_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Ident => Focus::Passwd,
            Focus::Passwd => Focus::Ident,
            Focus::None => Focus::Ident,
        };
    }

    pub fn lose_focus(&mut self) {
        self.focus = Focus::None;
    }

    pub fn has_focus(&self) -> bool {
        !matches!(self.focus, Focus::None)
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    pub fn unset_error(&mut self) {
        self.error = None;
    }
}

impl Widget for &Login {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .spacing(1)
        .split(
            Layout::horizontal([
                Constraint::Percentage(30),
                Constraint::Min(50),
                Constraint::Percentage(30),
            ])
            .split(area)[1],
        );

        set_style(
            self.ident.clone(),
            " identifier ",
            matches!(self.focus, Focus::Ident),
        )
        .widget()
        .render(layout[1], buf);

        set_style(
            self.passwd.clone(),
            " password ",
            matches!(self.focus, Focus::Passwd),
        )
        .widget()
        .render(layout[2], buf);

        if let Some(err) = &self.error {
            Paragraph::new(err.as_str())
                .alignment(Alignment::Center)
                .render(layout[3], buf);
        }
    }
}

fn create_textarea(mask: bool) -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default().not_underlined());
    if mask {
        textarea.set_mask_char('·');
    }
    textarea
}

fn set_style<'a>(textarea: TextArea<'a>, title: &'a str, focus: bool) -> TextArea<'a> {
    let mut textarea = textarea;

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1));
    let block = if focus {
        block.blue().bold()
    } else {
        block.dim()
    };
    textarea.set_block(block);

    textarea.set_cursor_style(if focus {
        Style::default().reversed()
    } else {
        Style::default().hidden()
    });

    textarea
}
