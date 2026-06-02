use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use chrono::Local;
use futures_util::{SinkExt, StreamExt};
use rcgen::{CertificateParams, DnType, KeyPair};
use rust_embed::Embed;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::sync::Mutex as StdMutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{broadcast, Mutex};
use tokio_util::sync::CancellationToken;

#[derive(Embed)]
#[folder = "../web-client/"]
struct WebClientAssets;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub bin_tx: broadcast::Sender<Vec<u8>>,
    pub app_handle: AppHandle,
    pub online_count: Arc<AtomicUsize>,
    pub live_active: Arc<AtomicBool>,
    pub init_segment: Arc<Mutex<Option<Vec<u8>>>>,
    pub danmaku_log: Arc<StdMutex<Vec<String>>>,
}

fn generate_self_signed_cert() -> Result<(String, String), String> {
    let mut params = CertificateParams::default();
    params.distinguished_name.push(DnType::CommonName, "BulletComment");
    params.distinguished_name.push(DnType::OrganizationName, "BulletComment Local");

    let key_pair = KeyPair::generate().map_err(|e| format!("Key generation failed: {}", e))?;
    let cert = params.self_signed(&key_pair).map_err(|e| format!("Cert generation failed: {}", e))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    Ok((cert_pem, key_pem))
}

pub fn save_danmaku_log(app_handle: &AppHandle, log: &[String]) -> Result<String, String> {
    if log.is_empty() {
        return Ok("".into());
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let logs_dir = app_data_dir.join("logs");
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
    let (tx, _) = broadcast::channel::<String>(100);
    let (bin_tx, _) = broadcast::channel::<Vec<u8>>(512);
    let online_count = Arc::new(AtomicUsize::new(0));
    let live_active = Arc::new(AtomicBool::new(false));
    let init_segment = Arc::new(Mutex::new(None::<Vec<u8>>));

    let state = AppState {
        tx,
        bin_tx,
        app_handle: app_handle.clone(),
        online_count,
        live_active,
        init_segment,
        danmaku_log: danmaku_log.clone(),
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
    let handle_for_cleanup = app_handle.clone();

    tokio::spawn(async move {
        tokio::select! {
            _ = axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>()) => {},
            _ = cancel_token.cancelled() => {},
        }
        let log = log_for_cleanup.lock().unwrap();
        if !log.is_empty() {
            let _ = save_danmaku_log(&handle_for_cleanup, &log);
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
        "content": "已连接到弹幕服务器"
    }).to_string());

    if state.live_active.load(Ordering::Relaxed) {
        init_msgs.push(serde_json::json!({
            "type": "live_start",
            "content": "当前有直播正在进行"
        }).to_string());
    }

    for msg in init_msgs {
        let _ = sender.send(Message::Text(msg.into())).await;
    }

    if state.live_active.load(Ordering::Relaxed) {
        let init_seg = state.init_segment.lock().await;
        if let Some(data) = init_seg.as_ref() {
            let _ = sender.send(Message::Binary(data.clone().into())).await;
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
                    if sender.send(Message::Binary(data.into())).await.is_err() {
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
                        if parsed["type"] == "danmaku" {
                            let content = parsed["content"].as_str().unwrap_or("");
                            let nickname = parsed["nickname"].as_str().unwrap_or("");
                            let device = parsed["device"].as_str().unwrap_or("");
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
                        }
                        if parsed["type"] == "live_start" {
                            state_recv.live_active.store(true, Ordering::Relaxed);
                            *state_recv.init_segment.lock().await = None;
                            is_first_binary = true;
                        } else if parsed["type"] == "live_stop" {
                            state_recv.live_active.store(false, Ordering::Relaxed);
                            *state_recv.init_segment.lock().await = None;
                            let log_entries: Vec<String>;
                            {
                                let log = state_recv.danmaku_log.lock().unwrap();
                                log_entries = log.clone();
                            }
                            if !log_entries.is_empty() {
                                let _ = save_danmaku_log(&state_recv.app_handle, &log_entries);
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
                    let _ = state_recv.bin_tx.send(data.to_vec());
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
