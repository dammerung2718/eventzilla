pub mod testing;

use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

// Emit `Event`s from practically anywhere
// Aggregate them using a provided `reducer` that will construct a `State`
pub trait Store<E>: Sync + Send {
    fn add(&mut self, subscriber: UnboundedSender<E>);
    fn emit(&self, event: E);
    fn history(&self) -> Arc<Mutex<Vec<E>>>;
}
