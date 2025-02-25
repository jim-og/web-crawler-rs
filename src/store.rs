use std::collections::HashSet;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// A thread-safe store which only allows items to be inserted.
#[derive(Default)]
pub struct Store<T: Eq + Hash> {
    store: Arc<Mutex<HashSet<T>>>,
}

impl<T: Eq + Hash> Store<T> {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Add a value to the store.
    pub fn insert(&self, item: T) -> bool {
        let mut store = match self.store.lock().ok() {
            Some(store) => store,
            None => {
                eprintln!("DataStore lock poisioned");
                return false;
            }
        };
        store.insert(item)
    }
}

#[cfg(test)]
mod tests {
    use super::Store;

    #[test]
    fn insert() {
        let store = Store::new();
        assert!(store.insert("a"));
        assert!(store.insert("b"));
        assert!(!store.insert("a"));
    }
}
