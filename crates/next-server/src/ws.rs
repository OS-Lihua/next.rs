use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

pub type WsHandlerFn =
    Arc<dyn Fn(WsConnection) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct WsConnection {
    pub sender: WsSender,
    pub receiver: WsReceiver,
}

pub struct WsSender {
    tx: mpsc::UnboundedSender<Message>,
}

impl WsSender {
    pub fn send_text(&self, text: impl Into<String>) {
        let s: String = text.into();
        let _ = self.tx.send(Message::Text(s.into()));
    }

    pub fn send_binary(&self, data: Vec<u8>) {
        let _ = self.tx.send(Message::Binary(data.into()));
    }

    pub fn close(&self) {
        let _ = self.tx.send(Message::Close(None));
    }
}

impl Clone for WsSender {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub struct WsReceiver {
    rx: mpsc::UnboundedReceiver<WsMessage>,
}

impl WsReceiver {
    pub async fn next(&mut self) -> Option<WsMessage> {
        self.rx.recv().await
    }
}

pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Close,
}

pub struct WsRegistry {
    handlers: HashMap<String, WsHandlerFn>,
}

impl WsRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn on<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(WsConnection) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let wrapped: WsHandlerFn = Arc::new(move |conn| Box::pin(handler(conn)));
        self.handlers.insert(path.to_string(), wrapped);
    }

    pub fn get_handler(&self, path: &str) -> Option<&WsHandlerFn> {
        self.handlers.get(path)
    }

    pub fn has_route(&self, path: &str) -> bool {
        self.handlers.contains_key(path)
    }
}

impl Default for WsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn handle_ws_upgrade(
    req: hyper::Request<hyper::body::Incoming>,
    handler_fn: WsHandlerFn,
) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, hyper::Error> {
    use hyper::header::{
        CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE,
    };
    use hyper::StatusCode;

    let key = req
        .headers()
        .get(SEC_WEBSOCKET_KEY)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let has_upgrade = req
        .headers()
        .get(UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_lowercase().contains("websocket"))
        .unwrap_or(false);

    if !has_upgrade || key.is_none() {
        return Ok(hyper::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(http_body_util::Full::new(bytes::Bytes::from(
                "Not a WebSocket request",
            )))
            .unwrap());
    }

    let accept_key = compute_accept_key(key.as_deref().unwrap());

    let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel::<Message>();
    let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<WsMessage>();

    let sender = WsSender { tx: outgoing_tx };
    let receiver = WsReceiver { rx: incoming_rx };
    let conn = WsConnection { sender, receiver };

    tokio::spawn(async move {
        handler_fn(conn).await;
    });

    let _ = (incoming_tx, outgoing_rx);

    Ok(hyper::Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(UPGRADE, "websocket")
        .header(CONNECTION, "Upgrade")
        .header(SEC_WEBSOCKET_ACCEPT, accept_key)
        .header(SEC_WEBSOCKET_VERSION, "13")
        .body(http_body_util::Full::new(bytes::Bytes::new()))
        .unwrap())
}

pub fn compute_accept_key(key: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    data_encoding::BASE64.encode(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_registry() {
        let mut registry = WsRegistry::new();
        registry.on("/ws/chat", |mut conn| async move {
            while let Some(msg) = conn.receiver.next().await {
                match msg {
                    WsMessage::Text(text) => {
                        conn.sender.send_text(format!("echo: {}", text));
                    }
                    WsMessage::Close => break,
                    _ => {}
                }
            }
        });

        assert!(registry.has_route("/ws/chat"));
        assert!(!registry.has_route("/ws/other"));
    }

    #[test]
    fn test_ws_sender_clone() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let sender = WsSender { tx };
        let sender2 = sender.clone();
        sender.send_text("hello");
        sender2.send_text("world");
    }

    #[test]
    fn test_compute_accept_key() {
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = compute_accept_key(key);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }
}
