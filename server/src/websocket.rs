//! WebSocket hub: JWT auth, join chat rooms, fan-out new messages to members

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::api::AppState;
use crate::auth::verify_token;

#[derive(Clone, Default)]
pub struct WsHub {
    inner: Arc<RwLock<HubInner>>,
}

struct HubInner {
    rooms: HashMap<String, broadcast::Sender<String>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HubInner {
                rooms: HashMap::new(),
            })),
        }
    }

    pub async fn subscribe(&self, chat_id: &str) -> broadcast::Receiver<String> {
        let mut g = self.inner.write().await;
        let tx = g
            .rooms
            .entry(chat_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            })
            .clone();
        tx.subscribe()
    }

    pub async fn publish(&self, chat_id: &str, payload: String) {
        let g = self.inner.read().await;
        if let Some(tx) = g.rooms.get(chat_id) {
            let _ = tx.send(payload);
        }
    }
}

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(q): Query<WsQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let claims = match q.token.as_deref() {
        Some(t) => verify_token(t, &state.jwt_secret).ok(),
        None => None,
    };
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims.map(|c| c.sub)))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: Option<String>) {
    let (mut tx, mut rx) = socket.split();
    let welcome = serde_json::json!({
        "type": "welcome",
        "service": "liberty-messenger",
        "auth": user_id.is_some(),
    });
    let _ = tx.send(Message::Text(welcome.to_string().into())).await;

    let mut joined: HashSet<String> = HashSet::new();
    let mut room_rx: Option<broadcast::Receiver<String>> = None;

    loop {
        tokio::select! {
            msg = rx.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) {
                            match v.get("type").and_then(|x| x.as_str()) {
                                Some("join") => {
                                    if let Some(chat_id) = v.get("chat_id").and_then(|x| x.as_str()) {
                                        joined.insert(chat_id.to_string());
                                        room_rx = Some(state.hub.subscribe(chat_id).await);
                                        let ok = serde_json::json!({"type":"joined","chat_id": chat_id});
                                        if tx.send(Message::Text(ok.to_string().into())).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                Some("ping") => {
                                    let _ = tx.send(Message::Text(r#"{"type":"pong"}"#.into())).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            room_msg = async {
                match room_rx.as_mut() {
                    Some(r) => r.recv().await.ok(),
                    None => std::future::pending::<Option<String>>().await,
                }
            } => {
                if let Some(payload) = room_msg {
                    if tx.send(Message::Text(payload.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
    let _ = joined;
}
