use super::Store;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

pub struct TestingStore<E> {
    events: Arc<Mutex<Vec<E>>>,
    subscribers: Vec<UnboundedSender<E>>,
}

impl<E> TestingStore<E> {
    pub fn new() -> TestingStore<E> {
        Self {
            events: Default::default(),
            subscribers: Default::default(),
        }
    }
}

impl<E> Store<E> for TestingStore<E>
where
    E: Clone + Send + 'static,
{
    fn add(&mut self, subscriber: UnboundedSender<E>) {
        self.subscribers.push(subscriber);
    }

    fn emit(&self, event: E) {
        for s in self.subscribers.iter() {
            _ = s.send(event.clone());
        }

        let events = self.events.clone();
        tokio::spawn(async move {
            events.lock().await.push(event);
        });
    }

    fn history(&self) -> Arc<Mutex<Vec<E>>> {
        self.events.clone()
    }
}
