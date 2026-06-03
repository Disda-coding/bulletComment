use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use bytes::Bytes;
use chrono::Local;
use futures_util::{SinkExt, StreamExt};
use rcgen::{CertificateParams, DnType, KeyPair};
use rust_embed::Embed;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::sync::Mutex as StdMutex;
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, Mutex};
use tokio_util::sync::CancellationToken;

const MAX_HISTORY: usize = 100;

#[derive(Embed)]
#[folder = "../web-client/"]
struct WebClientAssets;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VoteOption {
    pub label: String,
    pub count: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Vote {
    pub id: String,
    pub question: String,
    pub options: Vec<VoteOption>,
    pub voters: HashMap<String, usize>,
    #[serde(default)]
    pub vote_times: HashMap<String, i64>,
    pub created_by: String,
    pub created_at: String,
    pub closed: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub bin_tx: broadcast::Sender<Bytes>,
    pub app_handle: AppHandle,
    pub online_count: Arc<AtomicUsize>,
    pub live_active: Arc<AtomicBool>,
    pub init_segment: Arc<Mutex<Option<Vec<u8>>>>,
    pub danmaku_log: Arc<StdMutex<Vec<String>>>,
    pub danmaku_history: Arc<StdMutex<Vec<String>>>,
    pub active_votes: Arc<StdMutex<Vec<Vote>>>,
}

fn generate_self_signed_cert() -> Result<(String, String), String> {
    let mut params = CertificateParams::default();
    params.distinguished_name.push(DnType::CommonName, "BulletComment");
    params.distinguished_name.push(DnType::OrganizationName, "BulletComment Local");

    let key_pair = KeyPair::generate().map_err(|e| format!("Key generation failed: {}", e))?;
    let cert = params.self_signed(&key_pair).map_err(|e| format!("Cert generation failed: {}", e))?;

    Ok((cert.pem(), key_pair.serialize_pem()))
}

const LOG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../logs");

pub fn save_danmaku_log(log: &[String]) -> Result<String, String> {
    if log.is_empty() {
        return Ok("".into());
    }

    let logs_dir = std::path::PathBuf::from(LOG_DIR);
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("Failed to create logs dir: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("danmaku_{}.log", timestamp);
    let filepath = logs_dir.join(&filename);

    let header = format!(
        "=== 弹幕日志 {} ===\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    let content = header + &log.join("\n") + "\n";

    std::fs::write(&filepath, content)
        .map_err(|e| format!("Failed to write log file: {}", e))?;

    Ok(filepath.to_string_lossy().to_string())
}

pub async fn start_server(
    port: u16,
    app_handle: AppHandle,
    cancel_token: CancellationToken,
    danmaku_log: Arc<StdMutex<Vec<String>>>,
) -> Result<(), String> {
    let (tx, _) = broadcast::channel::<String>(256);
    let (bin_tx, _) = broadcast::channel::<Bytes>(512);
    let online_count = Arc::new(AtomicUsize::new(0));
    let live_active = Arc::new(AtomicBool::new(false));
    let init_segment = Arc::new(Mutex::new(None::<Vec<u8>>));
    let danmaku_history = Arc::new(StdMutex::new(Vec::new()));
    let active_votes = Arc::new(StdMutex::new(Vec::new()));

    let state = AppState {
        tx,
        bin_tx,
        app_handle: app_handle.clone(),
        online_count,
        live_active,
        init_segment,
        danmaku_log: danmaku_log.clone(),
        danmaku_history,
        active_votes,
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let (cert_pem, key_pem) = generate_self_signed_cert()?;

    let tls_config = RustlsConfig::from_pem(cert_pem.into_bytes(), key_pem.into_bytes())
        .await
        .map_err(|e| format!("TLS config failed: {}", e))?;

    let log_for_cleanup = danmaku_log.clone();

    tokio::spawn(async move {
        tokio::select! {
            _ = axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>()) => {},
            _ = cancel_token.cancelled() => {},
        }
        let log = log_for_cleanup.lock().unwrap_or_else(|e| e.into_inner());
        if !log.is_empty() {
            let _ = save_danmaku_log(&log);
        }
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();
    ws.on_upgrade(move |socket| handle_socket(socket, state, ip))
}

async fn handle_socket(socket: WebSocket, state: AppState, client_ip: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();
    let mut bin_rx = state.bin_tx.subscribe();

    state.online_count.fetch_add(1, Ordering::Relaxed);
    let count = state.online_count.load(Ordering::Relaxed);
    let _ = state.app_handle.emit("online-count", count);
    broadcast_online_count(&state);

    let mut init_msgs = Vec::new();

    init_msgs.push(serde_json::json!({
        "type": "system",
        "action": "connected",
        "content": "已连接到弹幕服务器",
        "ip": client_ip
    }).to_string());

    if state.live_active.load(Ordering::Relaxed) {
        init_msgs.push(serde_json::json!({
            "type": "live_start",
            "content": "当前有直播正在进行"
        }).to_string());
    }

    {
        let history = state.danmaku_history.lock().unwrap_or_else(|e| e.into_inner());
        for msg in history.iter() {
            init_msgs.push(msg.clone());
        }
    }

    {
        let votes = state.active_votes.lock().unwrap_or_else(|e| e.into_inner());
        for vote in votes.iter() {
            init_msgs.push(serde_json::json!({
                "type": "vote_create",
                "vote": vote
            }).to_string());
        }
    }

    for msg in init_msgs {
        let _ = sender.send(Message::Text(msg.into())).await;
    }

    if state.live_active.load(Ordering::Relaxed) {
        let init_seg = state.init_segment.lock().await;
        if let Some(data) = init_seg.as_ref() {
            let _ = sender.send(Message::Binary(Bytes::copy_from_slice(data))).await;
        }
    }

    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(msg) = rx.recv() => {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                Ok(data) = bin_rx.recv() => {
                    if sender.send(Message::Binary(data)).await.is_err() {
                        break;
                    }
                }
                else => { break; }
            }
        }
    });

    let state_recv = state.clone();
    let ip_for_log = client_ip.clone();
    let recv_task = tokio::spawn(async move {
        let mut is_first_binary = true;
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let msg_type = parsed["type"].as_str().unwrap_or("");

                        if msg_type == "danmaku" {
                            let content = parsed["content"].as_str().unwrap_or("");
                            let nickname = parsed["nickname"].as_str().unwrap_or("");
                            let device = parsed["device"].as_str().unwrap_or("");
                            let _color = parsed["color"].as_str().unwrap_or("#FFFFFF");
                            let time = Local::now().format("%H:%M:%S").to_string();
                            let sender_name = if !nickname.is_empty() {
                                format!("{} ({})", nickname, ip_for_log)
                            } else if !device.is_empty() {
                                format!("{} / {}", ip_for_log, device)
                            } else {
                                ip_for_log.clone()
                            };
                            let log_entry = format!("[{}] [{}] {}", time, sender_name, content);
                            if let Ok(mut log) = state_recv.danmaku_log.lock() {
                                log.push(log_entry);
                            }
                            if let Ok(mut history) = state_recv.danmaku_history.lock() {
                                history.push(text.to_string());
                                if history.len() > MAX_HISTORY {
                                    history.remove(0);
                                }
                            }
                        } else if msg_type == "vote_create" {
                            if let Ok(vote_data) = serde_json::from_value::<Vote>(parsed["vote"].clone()) {
                                if let Ok(mut votes) = state_recv.active_votes.lock() {
                                    votes.push(vote_data);
                                }
                            }
                        } else if msg_type == "vote_cast" {
                            let vote_id = parsed["vote_id"].as_str().unwrap_or("");
                            let option_idx = parsed["option_idx"].as_u64().map(|v| v as usize);
                            let voter = parsed["voter"].as_str().unwrap_or(&ip_for_log);
                            let now = chrono::Utc::now().timestamp_millis();
                            if let (Some(idx), Ok(mut votes)) = (option_idx, state_recv.active_votes.lock()) {
                                if let Some(vote) = votes.iter_mut().find(|v| v.id == vote_id) {
                                    if !vote.closed && idx < vote.options.len() {
                                        if let Some(&old_idx) = vote.voters.get(voter) {
                                            let can_change = vote.vote_times.get(voter)
                                                .map(|&t| now - t <= 5000)
                                                .unwrap_or(false);
                                            if can_change && old_idx != idx {
                                                vote.options[old_idx].count = vote.options[old_idx].count.saturating_sub(1);
                                                vote.options[idx].count += 1;
                                                vote.voters.insert(voter.to_string(), idx);
                                                vote.vote_times.insert(voter.to_string(), now);
                                                let updated = serde_json::json!({
                                                    "type": "vote_update",
                                                    "vote": vote
                                                }).to_string();
                                                let _ = state_recv.tx.send(updated);
                                            }
                                        } else {
                                            vote.voters.insert(voter.to_string(), idx);
                                            vote.vote_times.insert(voter.to_string(), now);
                                            vote.options[idx].count += 1;
                                            let updated = serde_json::json!({
                                                "type": "vote_update",
                                                "vote": vote
                                            }).to_string();
                                            let _ = state_recv.tx.send(updated);
                                        }
                                    }
                                }
                            }
                            continue;
                        } else if msg_type == "vote_close" {
                            let vote_id = parsed["vote_id"].as_str().unwrap_or("");
                            if let Ok(mut votes) = state_recv.active_votes.lock() {
                                if let Some(vote) = votes.iter_mut().find(|v| v.id == vote_id) {
                                    vote.closed = true;
                                    let updated = serde_json::json!({
                                        "type": "vote_update",
                                        "vote": vote
                                    }).to_string();
                                    let _ = state_recv.tx.send(updated);
                                }
                            }
                            continue;
                        }

                        if msg_type == "live_start" {
                            state_recv.live_active.store(true, Ordering::Relaxed);
                            *state_recv.init_segment.lock().await = None;
                            is_first_binary = true;
                        } else if msg_type == "live_stop" {
                            state_recv.live_active.store(false, Ordering::Relaxed);
                            *state_recv.init_segment.lock().await = None;
                            let log_entries: Vec<String>;
                            {
                                let log = state_recv.danmaku_log.lock().unwrap_or_else(|e| e.into_inner());
                                log_entries = log.clone();
                            }
                            if !log_entries.is_empty() {
                                let _ = save_danmaku_log(&log_entries);
                            }
                        }
                    }
                    let _ = state_recv.tx.send(text.to_string());
                    let _ = state_recv.app_handle.emit("danmaku", text.to_string());
                }
                Message::Binary(data) => {
                    if is_first_binary {
                        *state_recv.init_segment.lock().await = Some(data.to_vec());
                        is_first_binary = false;
                    }
                    let _ = state_recv.bin_tx.send(Bytes::copy_from_slice(&data));
                }
                _ => {}
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
