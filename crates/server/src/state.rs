use crate::session::SessionStore;
use shawk_common::WsEvent;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub events: broadcast::Sender<WsEvent>,
    /// Simple shared-secret admin auth for probe registration. Set via ADMIN_TOKEN env var.
    pub admin_token: String,
    pub sessions: std::sync::Arc<SessionStore>,
    /// Host group whose members may log into the dashboard.
    pub required_group: String,
    /// Open control channels to currently-connected probes, keyed by probe id.
    pub probe_control: std::sync::Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<String>>>>,
    /// Pending terminal handoffs: a dashboard session waiting for the probe
    /// to dial back in with the matching terminal socket.
    pub terminal_waiters:
        std::sync::Arc<RwLock<HashMap<Uuid, oneshot::Sender<axum::extract::ws::WebSocket>>>>,
}

impl AppState {
    pub fn new(pool: PgPool, admin_token: String, required_group: String) -> Self {
        let (tx, _rx) = broadcast::channel(1024);
        Self {
            pool,
            events: tx,
            admin_token,
            sessions: std::sync::Arc::new(SessionStore::new()),
            required_group,
            probe_control: std::sync::Arc::new(RwLock::new(HashMap::new())),
            terminal_waiters: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
