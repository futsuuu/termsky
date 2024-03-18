mod app;
mod atp;
mod tui;
mod utils;
mod view;
mod widgets;

use anyhow::Result;
use tokio::sync::mpsc;

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

    let mut atp = atp::Atp::new(atp_rx, res_tx.clone())?;
    let mut tui = tui::Tui::new(tui_rx, res_tx)?;
    tokio::try_join!(atp.start(), tui.start(), app::start(res_rx, atp_tx, tui_tx))?;

    Ok(())
}
