use crossterm::event::Event as TuiEvent;
use tokio::sync::mpsc;
use atrium_api::agent::Session;

pub type EventTx = mpsc::UnboundedSender<Event>;
pub type EventRx = mpsc::UnboundedReceiver<Event>;

pub fn new_channel() -> (EventTx, EventRx) {
    mpsc::unbounded_channel()
}

#[derive(Eq, PartialEq)]
pub enum Event {
    Tick,
    Tui(TuiEvent),
    GetSession(Option<Session>),
}
