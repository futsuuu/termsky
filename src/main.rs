mod app;
mod atp;
mod tui;
mod utils;
mod view;

use anyhow::Result;
use tokio::{task, sync::mpsc};

fn main() -> Result<()> {
    utils::init()?;
    scopeguard::defer! {
        tui::exit().expect("failed to reset the terminal");
    }
    tui::enter()?;
    main_async()
}

#[tokio::main]
async fn main_async() -> Result<()> {
    let (res_tx, res_rx) = mpsc::unbounded_channel();
    let (tui_tx, tui_rx) = mpsc::unbounded_channel();
    let (atp_tx, atp_rx) = mpsc::unbounded_channel();
    let tui_task = task::spawn(tui::handler(tui_rx, res_tx.clone()));
    let atp_task = task::spawn(atp::handler(atp_rx, res_tx));

    tokio::try_join!(
        app::start(res_rx, atp_tx, tui_tx),
        flatten(tui_task),
        flatten(atp_task),
    )?;

    Ok(())
}

async fn flatten<T>(task: task::JoinHandle<Result<T>>) -> Result<T> {
    Ok(task.await??)
}
