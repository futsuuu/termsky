use std::{
    io::{stdout, Stdout},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self as tui_event, EventStream, KeyEvent, MouseEvent},
    execute, queue, terminal,
};
use futures_util::{FutureExt, StreamExt};
use ratatui::prelude::*;
use tokio::{sync::mpsc, task, time};
use tracing::{event, Level};

use crate::view::View;

pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    rx: mpsc::UnboundedReceiver<Event>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: {
                let backend = CrosstermBackend::new(stdout());
                let mut terminal = Terminal::new(backend)?;
                terminal.clear()?;
                terminal
            },
            rx: {
                let (tx, rx) = mpsc::unbounded_channel();
                task::spawn(collect_event(tx));
                rx
            },
        })
    }

    #[inline]
    pub async fn event(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    #[inline]
    pub fn render(&mut self, view: &View) -> Result<()> {
        self.terminal.draw(|f| f.render_widget(view, f.size()))?;
        Ok(())
    }
}

impl From<Event> for tui_textarea::Input {
    fn from(value: Event) -> Self {
        match value {
            Event::Key(key) => Self::from(key),
            Event::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

async fn collect_event(tx: mpsc::UnboundedSender<Event>) {
    let mut interval = time::interval(Duration::from_millis(250));
    let mut events = EventStream::new();
    loop {
        let event = tokio::select! {
            _ = tx.closed() => {
                break;
            }
            _ = interval.tick() => {
                Some(Event::Tick)
            }
            Some(Ok(event)) = events.next().fuse() => {
                match event {
                    tui_event::Event::Key(key) => {
                        Some(Event::Key(key))
                    }
                    tui_event::Event::Mouse(mouse) => {
                        Some(Event::Mouse(mouse))
                    }
                    _ => None,
                }
            }
        };
        let Some(event) = event else {
            continue;
        };
        if tx.send(event).is_err() {
            break;
        }
    }

    event!(Level::DEBUG, "stop handler: channel closed");
}

pub fn enter() -> Result<()> {
    event!(Level::TRACE, "start rendering");
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
    event!(Level::TRACE, "finish rendering");
    Ok(())
}
