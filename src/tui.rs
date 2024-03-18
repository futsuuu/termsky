use std::{
    fmt,
    io::{stdout, Stdout},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{bail, Result};
use crossterm::{
    cursor,
    event::{self as tui_event, EventStream},
    execute, queue, terminal,
};
use futures_util::{FutureExt, StreamExt};
use ratatui::prelude::*;
use tokio::{sync::mpsc, task, time};
use tracing::{event, instrument, Level};

use crate::{app, view::View};

#[derive(Debug)]
pub enum Request {
    GetEvent,
    Render(View),
}

pub enum Response {
    Tick,
    Event(tui_event::Event),
}

pub struct Tui {
    req_rx: mpsc::UnboundedReceiver<Request>,
    res_tx: mpsc::UnboundedSender<app::Response>,
    terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
    interval: time::Interval,
    events: EventStream,
}

impl Tui {
    pub fn new(
        req_rx: mpsc::UnboundedReceiver<Request>,
        res_tx: mpsc::UnboundedSender<app::Response>,
    ) -> Result<Self> {
        let interval = time::interval(Duration::from_millis(250));
        let events = EventStream::new();
        let terminal = Arc::new(Mutex::new({
            let backend = CrosstermBackend::new(stdout());
            let mut terminal = Terminal::new(backend)?;
            terminal.clear()?;
            terminal
        }));
        Ok(Self {
            req_rx,
            res_tx,
            terminal,
            interval,
            events,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        while let Some(req) = self.req_rx.recv().await {
            let Ok(Some(res)) = self.handle_request(req).await else {
                continue;
            };
            if self.res_tx.send(app::Response::Tui(res)).is_err() {
                break;
            }
        }
        event!(Level::DEBUG, "stop handler: channel closed");
        Ok(())
    }

    #[instrument(name = "tui", err(level = Level::WARN), ret(level = Level::TRACE), skip(self))]
    async fn handle_request(&mut self, request: Request) -> Result<Option<Response>> {
        let res = match request {
            Request::GetEvent => tokio::select! {
                _ = self.res_tx.closed() => {
                    None
                }
                _ = self.interval.tick() => {
                    Some(Response::Tick)
                }
                Some(Ok(event)) = self.events.next().fuse() => {
                    Some(Response::Event(event))
                }
            },
            Request::Render(view) => {
                self.render(view)?;
                None
            }
        };
        Ok(res)
    }

    fn render(&self, view: View) -> Result<()> {
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

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Tick => f.write_str("Tick"),
            Self::Event(e) => {
                use tui_event::Event::*;
                write!(
                    f,
                    "Event({})",
                    match e {
                        FocusGained => "FocusGained",
                        FocusLost => "FocusLost",
                        Key(_) => "Key",
                        Mouse(_) => "Mouse",
                        Paste(_) => "Paste",
                        Resize(_, _) => "Paste",
                    }
                )
            }
        }
    }
}
