use std::{io::stdout, time::Duration};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self as tui_event, EventStream},
    execute, queue, terminal,
};
use futures_util::{FutureExt, StreamExt};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use crate::app;

pub enum Request {
    GetEvent,
    Render(crate::view::View),
}

pub enum Response {
    Tick,
    Event(tui_event::Event),
}

pub async fn handler(
    mut rx: mpsc::UnboundedReceiver<Request>,
    tx: mpsc::UnboundedSender<app::Response>,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(250));
    let mut events = EventStream::new();
    let mut terminal = {
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        terminal
    };

    while let Some(request) = rx.recv().await {
        let res = match &request {
            Request::GetEvent => Some(tokio::select! {
                _ = tx.closed() => {
                    break;
                }
                _ = interval.tick() => {
                    Response::Tick
                }
                    Some(Ok(event)) = events.next().fuse() => {
                    Response::Event(event)
                }
            }),
            Request::Render(view) => {
                terminal.draw(|f| view.render(f))?;
                None
            }
        };

        if let Some(res) = res {
            if tx.send(app::Response::Tui(res)).is_err() {
                break;
            }
        }
    }

    Ok(())
}

pub fn enter() -> Result<()> {
    terminal::enable_raw_mode()?;
    queue!(
        stdout(),
        terminal::EnterAlternateScreen,
        tui_event::EnableMouseCapture,
        cursor::Hide,
    )?;
    Ok(())
}

pub fn exit() -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(
        stdout(),
        terminal::LeaveAlternateScreen,
        tui_event::DisableMouseCapture,
        cursor::Show,
    )?;
    Ok(())
}
