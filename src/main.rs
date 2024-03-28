mod app;
mod atp;
mod tui;
mod utils;
mod view;
mod widgets;

use anyhow::Result;

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
    app::start().await?;
    Ok(())
}
