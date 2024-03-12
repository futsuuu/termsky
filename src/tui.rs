use std::{io::stdout, time::Duration};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self as tui_event, EventStream},
    execute, queue, terminal,
};
use futures_util::{FutureExt, StreamExt};
use ratatui::prelude::*;
use tracing::{event, Level};

use crate::{
    command::{Command, CommandRx},
    event::{Event, EventTx},
};

pub fn start(command_rx: CommandRx, event_tx: EventTx) {
    tokio::spawn(command_handler(command_rx));
    tokio::spawn(tui_event_handler(event_tx, 250));
}

async fn command_handler(mut command_rx: CommandRx) -> Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        if command_rx.changed().await.is_err() {
            event!(Level::WARN, "command channel is closed");
            break;
        }
        match *command_rx.borrow_and_update() {
            Command::Render(ref view) => {
                terminal.draw(|frame| view.render(frame))?;
            }
            _ => continue,
        }
    }

    Ok(())
}

async fn tui_event_handler(tx: EventTx, ms: u64) {
    let tick_rate = Duration::from_millis(ms);
    let mut tui_events = EventStream::new();
    let mut tick = tokio::time::interval(tick_rate);

    loop {
        let tick_delay = tick.tick();
        let tui_event = tui_events.next().fuse();
        let event = tokio::select! {
            _ = tx.closed() => {
                break;
            }
            _ = tick_delay => {
                Event::Tick
            }
            Some(Ok(tui_event)) = tui_event => {
                Event::Tui(tui_event)
            }
        };
        if tx.send(event).is_err() {
            break;
        }
    }
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
