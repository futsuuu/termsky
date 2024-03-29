use std::{
    cell::RefCell,
    io::{stdout, Stdout},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{bail, Result};
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
    terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
    rx: RefCell<mpsc::UnboundedReceiver<Event>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: {
                let backend = CrosstermBackend::new(stdout());
                let mut terminal = Terminal::new(backend)?;
                terminal.clear()?;
                Arc::new(Mutex::new(terminal))
            },
            rx: {
                let (tx, rx) = mpsc::unbounded_channel();
                task::spawn(collect_event(tx));
                RefCell::new(rx)
            },
        })
    }

    #[inline]
    pub async fn event(&self) -> Option<Event> {
        self.rx.borrow_mut().recv().await
    }

    #[inline]
    pub fn render(&self, view: View) -> Result<()> {
        // `TryLockError` is not `Send`, so cannot use anyhow directly
        if let Err(e) = self.terminal.try_lock() {
            bail!("skip rendering: {e}");
        }
        let terminal = Arc::clone(&self.terminal);
        let _task = task::spawn_blocking(move || -> Result<()> {
            let mut terminal = terminal.lock().unwrap();
            terminal.draw(|f| view.render(f))?;
            Ok(())
        });
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
