mod app;
mod atp;
mod command;
mod event;
mod tui;
mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    utils::init()?;
    scopeguard::defer! {
        tui::exit().expect("failed to reset the terminal");
    };
    tui::enter()?;

    // many producers to single consumer
    let (event_tx, event_rx) = event::new_channel();
    // single producer to many consumers
    let (command_tx, command_rx) = command::new_channel();

    tui::start(command_rx.clone(), event_tx.clone());
    atp::start(command_rx.clone(), event_tx.clone());

    app::start(event_rx, command_tx).await?;

    Ok(())
}
