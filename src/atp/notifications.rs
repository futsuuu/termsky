use std::sync::{Arc, Mutex};

use anyhow::Result;
use atrium_api::app::bsky::notification::list_notifications;

use super::Agent;

pub type Notification = Arc<list_notifications::Notification>;
pub type GetNotificationsResult = Result<Vec<Notification>>;

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

#[tracing::instrument(ret, err, skip(agent))]
async fn list_notifications(
    agent: Agent,
    params: list_notifications::Parameters,
) -> anyhow::Result<list_notifications::Output> {
    let notifications = agent
        .api
        .app
        .bsky
        .notification
        .list_notifications(params)
        .await?;
    anyhow::ensure!(
        notifications.cursor.is_some(),
        "cannot load more notifications"
    );
    Ok(notifications)
}
