use ratatui::{layout::Flex, prelude::*, widgets::*};

pub struct Tabs {
    pub tabs: Vec<Tab>,
    pub selected: usize,
}

pub struct Tab {
    text: String,
    active: bool,
}

impl Tab {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            active: true,
        }
    }
}

impl WidgetRef for Tabs {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let layouts = Layout::vertical(self.tabs.iter().map(|_| Constraint::Length(4))).split(area);
        for (i, (tab, area)) in self.tabs.iter().zip(layouts.iter()).enumerate() {
            let selected = i == self.selected;
            let area = Rect {
                height: area.height + 1,
                ..*area
            };

            let block = Block::new();
            let block = if selected {
                block
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().blue())
            } else {
                block
            };

            let [inner] = Layout::vertical([Constraint::Length(1)])
                .horizontal_margin(3)
                .flex(Flex::Center)
                .areas(area);
            Paragraph::new(tab.text.as_str())
                .style(if selected {
                    Style::new().bold()
                } else if tab.active {
                    Style::new()
                } else {
                    Style::new().dim()
                })
                .render(inner, buf);
            block.render(area, buf);
        }
    }
}
