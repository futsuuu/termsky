use std::{fs::create_dir_all, path::PathBuf};

use anyhow::{Context, Result};

pub fn init() -> Result<()> {
    create_dir_all(local_data_dir()?)?;
    Ok(())
}

pub fn local_data_dir() -> Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .context("failed to get the local data directory")?
        .join(env!("CARGO_PKG_NAME")))
}
