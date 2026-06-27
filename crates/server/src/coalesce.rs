use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, Weak};

use tokio::sync::broadcast;

pub type Key = (i64, i64, i32);

#[derive(Clone, Debug)]
pub struct CoalesceError(pub String);

type Sender<T> = broadcast::Sender<Result<T, CoalesceError>>;

enum Slot<T: Clone> {
    Lead(Arc<Sender<T>>),
    Follow(broadcast::Receiver<Result<T, CoalesceError>>),
}

pub struct Coalescer<T: Clone + Send + 'static> {
    inflight: Mutex<HashMap<Key, Weak<Sender<T>>>>,
}

impl<T: Clone + Send + 'static> Coalescer<T> {
    pub fn new() -> Self {
        Self {
            inflight: Mutex::new(HashMap::new()),
        }
    }

    fn acquire(&self, key: Key) -> Slot<T> {
        let mut map = self.inflight.lock().unwrap();
        match map.get(&key).and_then(Weak::upgrade) {
            Some(tx) => Slot::Follow(tx.subscribe()),
            None => {
                let (tx, _) = broadcast::channel(1);
                let tx = Arc::new(tx);
                map.insert(key, Arc::downgrade(&tx));
                Slot::Lead(tx)
            }
        }
    }

    pub async fn run<F, Fut>(&self, key: Key, f: F) -> Result<T, CoalesceError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CoalesceError>>,
    {
        match self.acquire(key) {
            Slot::Lead(tx) => {
                let result = f().await;
                self.inflight.lock().unwrap().remove(&key);
                let _ = tx.send(result.clone());
                result
            }
            Slot::Follow(mut rx) => match rx.recv().await {
                Ok(v) => v,
                Err(_) => Err(CoalesceError("leader dropped before completing".into())),
            },
        }
    }
}
