pub mod session;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::error::{NovaError, NovaResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

pub use session::TransportSession;

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, SocketAddr>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, session_id: Uuid, addr: SocketAddr) {
        let mut map = self.sessions.write().await;
        map.insert(session_id, addr);
    }

    pub async fn unregister(&self, session_id: &Uuid) {
        let mut map = self.sessions.write().await;
        map.remove(session_id);
    }

    pub async fn count(&self) -> usize {
        let map = self.sessions.read().await;
        map.len()
    }
}
