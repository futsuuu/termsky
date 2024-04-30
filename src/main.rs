mod app;
mod atp;
mod prelude;
mod tui;
mod utils;
mod widgets;

fn main() -> anyhow::Result<()> {
    utils::init()?;
    tui::enter()?;
    app::run()?;
    tui::exit()?;
    Ok(())
}
