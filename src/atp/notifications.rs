use std::sync::{Arc, Mutex};

use atrium_api::app::bsky::notification::list_notifications;

use super::Agent;

pub type Notification = Arc<list_notifications::Notification>;
pub type GetNotificationsResult = Result<Vec<Notification>, GetNotificationsError>;

#[derive(thiserror::Error, Debug)]
pub enum GetNotificationsError {
    #[error(transparent)]
    ATrium(#[from] atrium_xrpc::error::Error<list_notifications::Error>),
    #[error("all notifications have been loaded")]
    EndOfNotification,
}

pub(super) struct Notifications {
    notifications: Mutex<Vec<Notification>>,
    cursor: Mutex<Option<String>>,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            notifications: Mutex::new(Vec::new()),
            cursor: Mutex::new(None),
        }
    }
}

impl Notifications {
    pub async fn get_old(&self, agent: Agent) -> GetNotificationsResult {
        let cursor = self.cursor.lock().unwrap().clone();
        if cursor.is_none() && !self.notifications.lock().unwrap().is_empty() {
            return Err(GetNotificationsError::EndOfNotification);
        }
        let r = list_notifications(
            agent,
            list_notifications::Parameters {
                cursor,
                limit: 40.try_into().ok(),
                seen_at: None,
            },
        )
        .await?;
        *self.cursor.lock().unwrap() = r.cursor;
        let notifications: Vec<_> = r.notifications.into_iter().map(Arc::new).collect();
        self.notifications
            .lock()
            .unwrap()
            .extend(notifications.clone());
        Ok(notifications)
    }
}

#[tracing::instrument(err, skip(agent))]
async fn list_notifications(
    agent: Agent,
    params: list_notifications::Parameters,
) -> atrium_xrpc::error::Result<list_notifications::Output, list_notifications::Error> {
    let notifications = agent
        .api
        .app
        .bsky
        .notification
        .list_notifications(params)
        .await?;
    tracing::info!("receive {} notifications", notifications.notifications.len());
    if notifications.cursor.is_none() {
        tracing::info!("all notifications have been received");
    }
    Ok(notifications)
}
