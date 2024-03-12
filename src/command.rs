use std::fmt;

use anyhow::Result;
use tokio::sync::watch;
use tracing::{event, Level};

#[derive(Clone)]
pub enum Command {
    Dummy,
    GetSession,
    Login { ident: String, passwd: String },
    Render(crate::view::View),
}

pub fn new_channel() -> (CommandTx, CommandRx) {
    let (tx, rx) = watch::channel(Command::Dummy);
    (CommandTx(tx), rx)
}

pub struct CommandTx(watch::Sender<Command>);
pub type CommandRx = watch::Receiver<Command>;

impl CommandTx {
    pub fn send(&self, command: Command) -> Result<()> {
        if matches!(&command, Command::GetSession | Command::Login { .. }) {
            event!(Level::INFO, "send command: {command:?}");
        }
        let result = self.0.send(command);
        if result.is_err() {
            event!(Level::WARN, "command channel is closed");
        }
        result?;
        Ok(())
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Dummy => f.write_str("Dummy"),
            Command::GetSession => f.write_str("GetSession"),
            Command::Login { ident, passwd: _ } => f
                .debug_struct("Login")
                .field("ident", &ident)
                .field("passwd", &"***")
                .finish(),
            Command::Render(view) => f.debug_tuple("Render").field(&view).finish(),
        }
    }
}
