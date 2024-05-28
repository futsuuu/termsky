mod app;
mod atp;
mod prelude;
mod tui;
mod utils;
mod widgets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::init()?;
    tui::enter()?;
    app::run().await?;
    tui::exit()?;
    Ok(())
}
