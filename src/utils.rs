use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
    sync::Mutex,
};

use anyhow::{Context, Result};
use tracing::{
    event,
    level_filters::{LevelFilter, STATIC_MAX_LEVEL},
    Level,
};

pub fn init() -> Result<()> {
    setup_tracing()?;
    create_dir_all(local_data_dir()?)?;
    Ok(())
}

pub fn local_data_dir() -> Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .context("failed to get the local data directory")?
        .join(env!("CARGO_PKG_NAME")))
}

fn setup_tracing() -> Result<()> {
    if STATIC_MAX_LEVEL == LevelFilter::OFF {
        return Ok(());
    }

    let file = File::create("debug.log")?;
    tracing_subscriber::fmt()
        .with_level(true)
        .with_writer(Mutex::new(file))
        .without_time()
        .with_max_level(Level::TRACE)
        .init();
    event!(Level::TRACE, "start tracing");

    Ok(())
}
