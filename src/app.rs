use anyhow::Result;
use crossterm::event::{Event as TuiEvent, KeyCode};
use ratatui::terminal::Frame;

use crate::{
    command::{Command, CommandTx},
    event::{Event, EventRx},
};

pub async fn start(mut event_rx: EventRx, command_tx: CommandTx) -> Result<()> {
    command_tx.send(Command::GetSession)?;
    let session = loop {
        if let Event::GetSession(session) = event_rx.recv().await.unwrap() {
            break session;
        }
    };
    let view = if session.is_some() {
        View::Home
    } else {
        View::Login {
            username: String::new(),
            password: String::new(),
        }
    };

    loop {
        command_tx.send(Command::Render(view.clone()))?;
        let Some(event) = event_rx.recv().await else {
            break;
        };
        if event == Event::Tui(TuiEvent::Key(KeyCode::Esc.into())) {
            break;
        }
    }

    Ok(())
}

#[derive(Clone)]
pub enum View {
    Login { username: String, password: String },
    Home,
}

impl View {
    pub fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            ratatui::widgets::block::Block::default().title("hello"),
            frame.size(),
        );
    }
}
