//! Shared application state for cross-handler and cross-middleware data sharing.
//!
//! Provides a thread-safe [`SharedState`] type that can be used to share data
//! across handlers and middleware in a concurrent environment.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe shared application state.
pub type SharedState = Arc<RwLock<StateMap>>;

/// A type-erased map for storing shared application state.
///
/// Values are stored as `Box<dyn Any + Send + Sync>` and can be retrieved
/// with type-safe downcasting.
///
/// # Example
/// ```
/// use mini_http::context::{new_shared_state};
///
/// let state = new_shared_state();
/// {
///     let mut s = state.write().unwrap();
///     s.insert("counter", 0u64);
/// }
/// {
///     let s = state.read().unwrap();
///     let counter = s.get::<u64>("counter").unwrap();
///     assert_eq!(*counter, 0);
/// }
/// ```
pub struct StateMap {
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl StateMap {
    /// Create a new empty state map.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert a typed value into the state map.
    pub fn insert<T: Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.data.insert(key.into(), Box::new(value));
    }

    /// Get an immutable reference to a typed value.
    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    /// Get a mutable reference to a typed value.
    pub fn get_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.data.get_mut(key).and_then(|v| v.downcast_mut::<T>())
    }

    /// Remove a value from the state map.
    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    /// Check if the state map contains a key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

impl Default for StateMap {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for StateMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMap")
            .field("keys", &self.data.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Create a new empty shared state.
pub fn new_shared_state() -> SharedState {
    Arc::new(RwLock::new(StateMap::new()))
}
