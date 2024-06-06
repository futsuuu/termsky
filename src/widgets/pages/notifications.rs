use std::cell::Cell;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::Block,
};

use crate::{
    app::App,
    atp::Response,
    widgets::{atoms::BlockExt, organisms::Notification, RectExt, Store, Storeable},
};

#[derive(Default)]
pub struct Notifications {
    scroll: u16,
    blank_height: Cell<Option<u16>>,
    notifications: Vec<Notification>,
    response: Response<crate::atp::GetNotificationsResult>,
}

impl Notifications {
    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll += 1;
    }
}

impl ratatui::widgets::WidgetRef for Notifications {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(5),
            Constraint::Fill(1),
        ])
        .areas(area);

        let mut store = Store::new().scroll_v(self.scroll as i32);
        for notification in &self.notifications {
            if notification.is_read() {
                Block::new().border_style(Style::new().blue().dim())
            } else {
                Block::new()
                    .border_style(Style::new().blue())
                    .border_set(ratatui::symbols::border::THICK)
            }
            .borders(ratatui::widgets::Borders::BOTTOM)
            .wrap_child(notification.clone())
            .fit_vertical()
            .store(store.bottom_space(area.height(u16::MAX)), &mut store);
        }
        self.blank_height
            .set((self.scroll + area.height).checked_sub(store.stored_area().height));
        store.render_ref(area, buf);
    }
}

impl crate::app::EventHandler for Notifications {
    fn on_render(&mut self, app: &mut App) {
        if self.blank_height.get().is_some() && self.response.is_empty() {
            self.response = app.atp.get_notifications();
        }

        if let Some(Ok(notifications)) = self.response.take_data() {
            for notification in notifications {
                self.notifications.push(Notification::from(notification));
            }
        }
    }

    fn on_key(&mut self, ev: crossterm::event::KeyEvent, app: &mut App) {
        if ev.code == KeyCode::Esc {
            app.exit();
            return;
        }
        if ev.code == KeyCode::Char('k') {
            self.scroll_up();
        }
        if ev.code == KeyCode::Char('j') {
            self.scroll_down();
        }
    }
}
