use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::Framed;
use uuid::Uuid;
use crate::error::{NovaError, NovaResult};
use crate::protocol::{
    EmptyPayload, MessageEnvelope, NovaFrameCodec, RawFrame, PROTOCOL_VERSION,
};
use crate::transport::ConnectionState;

pub struct TransportSession {
    pub session_id: Uuid,
    pub peer_device_id: Option<Uuid>,
    pub remote_addr: SocketAddr,
    pub state: ConnectionState,
    framed: Framed<TcpStream, NovaFrameCodec>,
    last_activity: Instant,
}

impl TransportSession {
    pub fn new(stream: TcpStream, remote_addr: SocketAddr) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            peer_device_id: None,
            remote_addr,
            state: ConnectionState::Connected,
            framed: Framed::new(stream, NovaFrameCodec::default()),
            last_activity: Instant::now(),
        }
    }

    pub async fn send_envelope<T: Serialize>(&mut self, envelope: &MessageEnvelope<T>) -> NovaResult<()> {
        let json_bytes = serde_json::to_vec(envelope)?;
        let frame = RawFrame::new(json_bytes);
        self.framed.send(frame).await?;
        self.last_activity = Instant::now();
        Ok(())
    }

    pub async fn send_message<T: Serialize>(
        &mut self,
        message_type: &str,
        payload: T,
    ) -> NovaResult<Uuid> {
        let envelope = MessageEnvelope::new(message_type, payload);
        let msg_id = envelope.message_id;
        self.send_envelope(&envelope).await?;
        Ok(msg_id)
    }

    pub async fn send_ping(&mut self) -> NovaResult<()> {
        self.send_message("ping", EmptyPayload {}).await?;
        Ok(())
    }

    pub async fn send_pong(&mut self, ping_id: Uuid) -> NovaResult<()> {
        let envelope = MessageEnvelope::new_reply("pong", ping_id, EmptyPayload {});
        self.send_envelope(&envelope).await?;
        Ok(())
    }

    pub async fn receive_raw_frame(&mut self) -> NovaResult<Option<RawFrame>> {
        match self.framed.next().await {
            Some(Ok(frame)) => {
                self.last_activity = Instant::now();
                Ok(Some(frame))
            }
            Some(Err(e)) => {
                self.state = ConnectionState::Failed;
                Err(e)
            }
            None => {
                self.state = ConnectionState::Disconnected;
                Ok(None)
            }
        }
    }

    pub fn is_idle(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
}
