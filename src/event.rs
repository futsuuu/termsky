use std::fmt;

use anyhow::Result;
use atrium_api::agent::Session;
use crossterm::event::Event as TuiEvent;
use tokio::sync::mpsc;
use tracing::{event, Level};

#[derive(Eq, PartialEq)]
pub enum Event {
    Tick,
    Tui(TuiEvent),
    Session(Option<Session>),
    Login { err: Option<String> },
}

pub fn new_channel() -> (EventTx, EventRx) {
    let (tx, rx) = mpsc::unbounded_channel();
    (EventTx(tx), rx)
}

#[derive(Clone)]
pub struct EventTx(mpsc::UnboundedSender<Event>);
pub type EventRx = mpsc::UnboundedReceiver<Event>;

impl EventTx {
    pub fn send(&self, event: Event) -> Result<()> {
        if matches!(&event, Event::Login { .. } | Event::Session(_)) {
            event!(Level::INFO, "send event: {event:?}");
        }
        let result = self.0.send(event);
        if result.is_err() {
            event!(Level::WARN, "event channel is closed");
        }
        result?;
        Ok(())
    }

    pub async fn closed(&self) {
        self.0.closed().await;
        event!(Level::WARN, "event channel is closed");
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::Tick => f.write_str("Tick"),
            Event::Tui(event) => f.debug_tuple("Tui").field(&event).finish(),
            Event::Session(session) => match session {
                None => f.write_str("Session(None)"),
                Some(session) => f.write_fmt(format_args!(
                    "Session(Some(Session {{ handle: \"{}\", .. }}))",
                    session.handle.as_str()
                )),
            },
            Event::Login { err } => f.debug_struct("Login").field("err", &err).finish(),
        }
    }
}

macro_rules! recv {
    ($event_rx:expr, $pattern:pat) => {
        loop {
            let Some(event) = $event_rx.recv().await else {
                break None;
            };
            if matches!(event, $pattern) {
                break Some(event);
            }
        }
    };
}
pub(crate) use recv;
