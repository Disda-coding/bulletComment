use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

#[derive(Embed)]
#[folder = "../web-client/"]
struct WebClientAssets;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DanmakuMessage {
    #[serde(rename = "type")]
    msg_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<usize>,
}

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub app_handle: AppHandle,
    pub online_count: Arc<AtomicUsize>,
}

pub async fn start_server(port: u16, app_handle: AppHandle) -> Result<(), String> {
    let (tx, _) = broadcast::channel::<String>(100);
    let online_count = Arc::new(AtomicUsize::new(0));

    let state = AppState {
        tx,
        app_handle,
        online_count,
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind: {}", e))?;

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    Ok(())
}

async fn serve_index() -> impl IntoResponse {
    let html = WebClientAssets::get("index.html")
        .map(|f| f.data.to_vec())
        .unwrap_or_default();
    (
        [("Content-Type", "text/html; charset=utf-8")],
        html,
    )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    state.online_count.fetch_add(1, Ordering::Relaxed);
    let count = state.online_count.load(Ordering::Relaxed);
    let _ = state.app_handle.emit("online-count", count);
    broadcast_online_count(&state);

    let sys_msg = serde_json::json!({
        "type": "system",
        "action": "connected",
        "content": "已连接到弹幕服务器"
    })
    .to_string();
    let _ = sender.send(Message::Text(sys_msg.into())).await;

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let state_recv = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                let _ = state_recv.tx.send(text.to_string());
                let _ = state_recv.app_handle.emit("danmaku", text.to_string());
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    state.online_count.fetch_sub(1, Ordering::Relaxed);
    let count = state.online_count.load(Ordering::Relaxed);
    let _ = state.app_handle.emit("online-count", count);
    broadcast_online_count(&state);
}

fn broadcast_online_count(state: &AppState) {
    let count = state.online_count.load(Ordering::Relaxed);
    let msg = serde_json::json!({
        "type": "online_count",
        "count": count
    })
    .to_string();
    let _ = state.tx.send(msg);
}
