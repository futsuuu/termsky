use ratatui::{prelude::*, widgets::*};

pub struct Spinner;

impl Spinner {
    pub fn new() -> Self {
        Self
    }
}

impl WidgetRef for Spinner {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let dot = "•";
        const N: usize = 5;

        let dots = {
            use std::time::SystemTime;

            let mut dots = vec![dot.dim().blue(); N];
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let i = (now / 250) % (N as u128 + 1);
            if i != N as u128 {
                dots[i as usize] = dot.bold();
            }
            dots
        };

        let [_, area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length((N * 2 + 1) as u16),
            Constraint::Fill(1),
        ])
        .areas(area);
        let layouts = Layout::horizontal([Constraint::Length(1); N])
            .spacing(1)
            .split(area);
        for (area, dot) in layouts.iter().zip(dots.iter()) {
            dot.render_ref(*area, buf);
        }
    }
}
