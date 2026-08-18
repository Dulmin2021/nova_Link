use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_util::codec::Framed;
use uuid::Uuid;
use crate::error::{NovaError, NovaResult};
use crate::protocol::{NovaFrameCodec, RawFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

pub struct PeerConnection {
    pub peer_id: Option<Uuid>,
    pub remote_addr: SocketAddr,
    pub state: ConnectionState,
    framed: Arc<Mutex<Framed<TcpStream, NovaFrameCodec>>>,
}

impl PeerConnection {
    pub fn new(stream: TcpStream, remote_addr: SocketAddr) -> Self {
        let framed = Framed::new(stream, NovaFrameCodec::default());
        Self {
            peer_id: None,
            remote_addr,
            state: ConnectionState::Connected,
            framed: Arc::new(Mutex::new(framed)),
        }
    }

    pub async fn connect(addr: SocketAddr) -> NovaResult<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self::new(stream, addr))
    }
}
