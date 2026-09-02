use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use std::{collections::HashMap, sync::RwLock};

pub const SESSION_COOKIE: &str = "shawk_session";
const SESSION_TTL_HOURS: i64 = 12;

#[derive(Clone)]
pub struct Session {
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

/// Simple in-memory session store. A server restart logs everyone out,
/// which is an acceptable tradeoff for a small self-hosted dashboard.
#[derive(Default)]
pub struct SessionStore {
    sessions: RwLock<HashMap<String, Session>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, username: &str) -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        let session = Session {
            username: username.to_string(),
            expires_at: Utc::now() + Duration::hours(SESSION_TTL_HOURS),
        };
        self.sessions.write().unwrap().insert(token.clone(), session);
        token
    }

    /// Returns the session's username if the token is valid and unexpired.
    pub fn validate(&self, token: &str) -> Option<String> {
        let sessions = self.sessions.read().unwrap();
        let session = sessions.get(token)?;
        if session.expires_at < Utc::now() {
            None
        } else {
            Some(session.username.clone())
        }
    }

    pub fn revoke(&self, token: &str) {
        self.sessions.write().unwrap().remove(token);
    }
}
