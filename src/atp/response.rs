use std::cell::RefCell;

use tokio::sync::oneshot;

/// A simple wrapper of [`tokio::sync::oneshot::Receiver`].
#[derive(Debug)]
pub struct Response<T> {
    inner: Option<RefCell<Inner<T>>>,
}

#[derive(Debug)]
struct Inner<T> {
    receiver: oneshot::Receiver<T>,
    received: Option<T>,
}

impl<T> Default for Response<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> From<oneshot::Receiver<T>> for Response<T> {
    fn from(receiver: oneshot::Receiver<T>) -> Self {
        let inner = Inner {
            receiver,
            received: None,
        };
        Self {
            inner: Some(RefCell::new(inner)),
        }
    }
}

impl<T: Send + 'static> Response<T> {
    /// Spawns an async task.
    pub fn new(future: impl std::future::Future<Output = T> + Send + 'static) -> Self {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async {
            tx.send(future.await).ok();
        });
        Self::from(rx)
    }
}

impl<T> Response<T> {
    /// Create an empty response.
    pub fn empty() -> Self {
        Self { inner: None }
    }

    /// Returns `true` if the response can never return data.
    pub fn is_empty(&self) -> bool {
        self.inner
            .as_ref()
            .map_or(true, |r| r.borrow_mut().is_empty())
    }

    /// Returns `true` until the task is running.
    pub fn is_loading(&self) -> bool {
        self.inner
            .as_ref()
            .map_or(false, |r| r.borrow_mut().is_loading())
    }

    /// If the data has been received, return it only once.
    /// After calling this method, the response will be empty.
    pub fn take_data(&self) -> Option<T> {
        self.inner.as_ref()?.borrow_mut().take_data()
    }
}

impl<T> Inner<T> {
    fn is_loading(&mut self) -> bool {
        if self.received.is_some() {
            return false;
        }
        match self.receiver.try_recv() {
            Ok(received) => {
                self.received = Some(received);
                false
            }
            Err(oneshot::error::TryRecvError::Empty) => true,
            Err(oneshot::error::TryRecvError::Closed) => false,
        }
    }

    fn is_empty(&mut self) -> bool {
        if self.received.is_some() {
            return false;
        }
        match self.receiver.try_recv() {
            Ok(received) => {
                self.received = Some(received);
                false
            }
            Err(oneshot::error::TryRecvError::Empty) => false,
            Err(oneshot::error::TryRecvError::Closed) => true,
        }
    }

    fn take_data(&mut self) -> Option<T> {
        self.received
            .take()
            .or_else(|| self.receiver.try_recv().ok())
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{sleep, Duration};

    use super::*;

    #[tokio::test]
    async fn take_data() {
        let res = Response::new(async {
            sleep(Duration::from_millis(10)).await;
            1
        });

        assert_eq!(None, res.take_data());

        sleep(Duration::from_millis(50)).await;
        assert_eq!(Some(1), res.take_data());

        assert_eq!(None, res.take_data());
    }

    #[tokio::test]
    async fn is_loading() {
        let res = Response::new(async {
            sleep(Duration::from_millis(10)).await;
        });

        assert!(res.is_loading());

        sleep(Duration::from_millis(50)).await;
        assert!(!res.is_loading());
    }

    #[tokio::test]
    async fn is_empty() {
        assert!(Response::<()>::empty().is_empty());

        let res = Response::new(async {
            sleep(Duration::from_millis(10)).await;
            1
        });

        assert!(!res.is_empty());

        sleep(Duration::from_millis(50)).await;
        assert!(!res.is_empty());

        res.take_data();
        assert!(res.is_empty());
    }
}
