//! In-memory session management.
//!
//! Provides a thread-safe session store with automatic session creation,
//! cookie-based session tracking, and configurable expiration.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::cookie::{CookieBuilder, SameSite};
use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::Response;

/// Session data for a single client.
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session identifier.
    pub id: String,
    /// Key-value session data.
    pub data: HashMap<String, String>,
    /// When this session was created.
    pub created_at: Instant,
    /// When this session was last accessed.
    pub last_accessed: Instant,
    /// Session lifetime.
    pub max_age: Duration,
}

impl Session {
    fn new(max_age: Duration) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4().to_string(),
            data: HashMap::new(),
            created_at: now,
            last_accessed: now,
            max_age,
        }
    }

    /// Get a session value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// Set a session value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Remove a session value.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    /// Clear all session data.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Check if the session has expired.
    pub fn is_expired(&self) -> bool {
        self.last_accessed.elapsed() > self.max_age
    }
}

/// Thread-safe in-memory session store.
///
/// # Example
/// ```
/// use mini_http::session::SessionStore;
/// use std::time::Duration;
///
/// let store = SessionStore::new()
///     .cookie_name("my_session")
///     .max_age(Duration::from_secs(7200));
/// ```
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    cookie_name: String,
    max_age: Duration,
    cookie_path: String,
    cookie_secure: bool,
    cookie_http_only: bool,
}

impl SessionStore {
    /// Create a new session store with default settings.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            cookie_name: "sid".to_string(),
            max_age: Duration::from_secs(3600),
            cookie_path: "/".to_string(),
            cookie_secure: false,
            cookie_http_only: true,
        }
    }

    /// Set the session cookie name.
    pub fn cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set the session lifetime.
    pub fn max_age(mut self, duration: Duration) -> Self {
        self.max_age = duration;
        self
    }

    /// Set the cookie path.
    pub fn cookie_path(mut self, path: impl Into<String>) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Set the cookie secure flag.
    pub fn cookie_secure(mut self, secure: bool) -> Self {
        self.cookie_secure = secure;
        self
    }

    /// Get an existing session by ID.
    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().unwrap();
        sessions
            .get(session_id)
            .filter(|s| !s.is_expired())
            .cloned()
    }

    /// Create a new session and store it.
    pub fn create_session(&self) -> Session {
        let session = Session::new(self.max_age);
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        session
    }

    /// Update an existing session in the store.
    pub fn save_session(&self, session: &Session) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id.clone(), session.clone());
    }

    /// Destroy a session by ID.
    pub fn destroy_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id);
    }

    /// Remove all expired sessions from the store.
    pub fn cleanup(&self) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, s| !s.is_expired());
    }

    /// Get the current number of active sessions.
    pub fn count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.values().filter(|s| !s.is_expired()).count()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for SessionStore {
    fn before(&self, req: &mut Request, res: &mut Response) -> bool {
        let session = req
            .cookie(&self.cookie_name)
            .and_then(|id| self.get_session(id))
            .unwrap_or_else(|| {
                let session = self.create_session();
                let mut cookie = CookieBuilder::new(&self.cookie_name, &session.id)
                    .path(&self.cookie_path)
                    .same_site(SameSite::Lax)
                    .max_age(self.max_age.as_secs() as i64);

                if self.cookie_http_only {
                    cookie = cookie.http_only();
                }
                if self.cookie_secure {
                    cookie = cookie.secure();
                }

                res.set_cookie(cookie);
                session
            });

        req.set_extension("session", session);
        true
    }

    fn after(&self, req: &mut Request, _res: &mut Response) {
        // Persist any session changes
        if let Some(session) = req.get_extension::<Session>("session") {
            self.save_session(session);
        }
    }

    fn name(&self) -> &str {
        "session"
    }
}
