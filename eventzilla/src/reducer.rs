use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    Mutex, OwnedMutexGuard,
};

pub type Data<T> = OwnedMutexGuard<T>;
type ReducerFn<T, E> = fn(&mut Data<T>, E);

#[derive(Clone)]
pub struct Reducer<T, E> {
    data: Arc<Mutex<T>>,
    reducer: ReducerFn<T, E>,
}

impl<T, E> Reducer<T, E>
where
    T: Clone + Default + Send + 'static,
    E: Send + 'static,
{
    pub fn new(reducer: ReducerFn<T, E>, init: T) -> Reducer<T, E> {
        Self {
            data: Arc::new(Mutex::new(init)),
            reducer,
        }
    }

    pub fn subscription(&self) -> UnboundedSender<E> {
        let (tx, mut rx) = unbounded_channel();
        let data = self.data.clone();
        let reducer = self.reducer;
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let mut data = data.clone().lock_owned().await;
                (reducer)(&mut data, event);
            }
        });
        tx
    }

    pub async fn read(&self) -> T {
        self.data.lock().await.clone()
    }
}
