use std::cell::RefCell;

use tokio::sync::oneshot;

/// A simple wrapper of [`tokio::sync::oneshot::Receiver`]
#[derive(Debug)]
pub struct Response<T> {
    inner: Option<RefCell<Inner<T>>>,
}

impl<T: Send + 'static> Response<T> {
    pub fn new(future: impl std::future::Future<Output = T> + Send + 'static) -> Self {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async {
            tx.send(future.await).ok();
        });
        Self::from(rx)
    }

    pub fn empty() -> Self {
        Self { inner: None }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_none()
    }

    pub fn is_loading(&self) -> bool {
        self.inner
            .as_ref()
            .map_or(false, |r| r.borrow_mut().is_loading())
    }

    // pub fn is_closed(&self) -> bool {
    //     self.inner
    //         .as_ref()
    //         .map_or(false, |r| r.borrow_mut().is_closed())
    // }

    pub fn take_data(&self) -> Option<T> {
        self.inner.as_ref()?.borrow_mut().take_data()
    }
}

impl<T> From<oneshot::Receiver<T>> for Response<T> {
    fn from(value: oneshot::Receiver<T>) -> Self {
        Self {
            inner: Some(RefCell::new(Inner::new(value))),
        }
    }
}

#[derive(Debug)]
struct Inner<T> {
    receiver: oneshot::Receiver<T>,
    received: Option<T>,
}

impl<T> Inner<T> {
    fn new(receiver: oneshot::Receiver<T>) -> Self {
        Self {
            receiver,
            received: None,
        }
    }

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

    // fn is_closed(&mut self) -> bool {
    //     !self.is_loading()
    // }

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
}
