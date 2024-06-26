use std::{
    io::{stdout, Result, Stdout},
    panic,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self as tui_event, EventStream, KeyEvent, KeyEventKind, MouseEvent},
    execute, queue, terminal,
};
use futures_util::{FutureExt, StreamExt};
use ratatui::prelude::*;
use tokio::{sync::mpsc, task, time};
use tracing::{event, Level};

use crate::prelude::*;

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
    event!(Level::TRACE, "start reading events");
    tx.send(Event::Tick).ok();
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
                        if key.kind == KeyEventKind::Release {
                            None
                        } else {
                            Some(Event::Key(key))
                        }
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

    event!(Level::TRACE, "stop reading events: channel closed");
}

pub fn enter() -> Result<()> {
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        exit().expect("failed to reset the terminal");
        panic_hook(panic);
    }));

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
