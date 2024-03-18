use std::path::PathBuf;

use anyhow::{bail, Result};
use async_trait::async_trait;
use atrium_api::agent::{store::SessionStore, Session};
use tokio::fs;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct FileStore(PathBuf);

impl FileStore {
    pub fn new() -> Result<Self> {
        Ok(Self(crate::utils::local_data_dir()?.join("session.json")))
    }

    async fn open_file(&self) -> Result<std::fs::File> {
        let file = fs::File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.0)
            .await?;
        Ok(file.into_std().await)
    }

    #[instrument(name = "get_session", err)]
    async fn get(&self) -> Result<Session> {
        if fs::try_exists(&self.0).await? {
            let session = serde_json::from_reader(self.open_file().await?)?;
            Ok(session)
        } else {
            bail!("file not found");
        }
    }

    #[instrument(name = "set_session", err, skip(session))]
    async fn set(&self, session: Session) -> Result<()> {
        serde_json::to_writer(self.open_file().await?, &session)?;
        Ok(())
    }

    #[instrument(name = "clear_session", err)]
    async fn clear(&self) -> Result<()> {
        fs::remove_file(&self.0).await?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for FileStore {
    async fn get_session(&self) -> Option<Session> {
        self.get().await.ok()
    }

    async fn set_session(&self, session: Session) {
        self.set(session).await.ok();
    }

    async fn clear_session(&self) {
        self.clear().await.ok();
    }
}
