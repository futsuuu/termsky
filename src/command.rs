use tokio::sync::watch;

pub type CommandTx = watch::Sender<Command>;
pub type CommandRx = watch::Receiver<Command>;

pub fn new_channel() -> (CommandTx, CommandRx) {
    watch::channel(Command::Dummy)
}

pub enum Command {
    Dummy,
    GetSession,
    Render(crate::app::View),
}
