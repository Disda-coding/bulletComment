use local_ip_address::local_ip;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio_util::sync::CancellationToken;

pub struct ServerState {
    pub running: AtomicBool,
    pub port: Mutex<u16>,
    pub pinned: AtomicBool,
    pub cancel_token: Mutex<Option<CancellationToken>>,
    pub danmaku_log: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

#[tauri::command]
pub async fn start_server(port: u16, app_handle: AppHandle) -> Result<String, String> {
    let state = app_handle.state::<ServerState>();

    if state.running.load(Ordering::SeqCst) {
        return Err("Server is already running".into());
    }

    {
        let mut log = state.danmaku_log.lock().map_err(|e| e.to_string())?;
        log.clear();
    }

    let cancel_token = CancellationToken::new();
    crate::server::start_server(
        port,
        app_handle.clone(),
        cancel_token.clone(),
        state.danmaku_log.clone(),
    )
    .await?;

    *state.cancel_token.lock().map_err(|e| e.to_string())? = Some(cancel_token);
    state.running.store(true, Ordering::SeqCst);
    *state.port.lock().map_err(|e| e.to_string())? = port;

    let ip = local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".into());
    let address = format!("https://{}:{}", ip, port);

    create_danmaku_window(&app_handle)?;
    create_control_bar_window(&app_handle)?;

    let _ = app_handle.emit("server-started", &address);

    Ok(address)
}

#[tauri::command]
pub async fn stop_server(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<ServerState>();

    if !state.running.load(Ordering::SeqCst) {
        return Err("Server is not running".into());
    }

    {
        let log = state.danmaku_log.lock().map_err(|e| e.to_string())?;
        if !log.is_empty() {
            let _ = crate::server::save_danmaku_log(&app_handle, &log);
        }
    }

    if let Some(token) = state.cancel_token.lock().ok().and_then(|mut t| t.take()) {
        token.cancel();
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    state.running.store(false, Ordering::SeqCst);

    if let Some(window) = app_handle.get_webview_window("danmaku") {
        let _ = window.close();
    }
    if let Some(window) = app_handle.get_webview_window("control-bar") {
        let _ = window.close();
    }

    let _ = app_handle.emit("server-stopped", ());

    Ok(())
}

#[tauri::command]
pub fn get_local_ip() -> Result<String, String> {
    local_ip()
        .map(|ip| ip.to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_server_status(app_handle: AppHandle) -> Result<bool, String> {
    let state = app_handle.state::<ServerState>();
    Ok(state.running.load(Ordering::SeqCst))
}

#[tauri::command]
pub fn toggle_pin(app_handle: AppHandle) -> Result<bool, String> {
    let state = app_handle.state::<ServerState>();
    let new_pinned = !state.pinned.load(Ordering::SeqCst);
    state.pinned.store(new_pinned, Ordering::SeqCst);

    if let Some(window) = app_handle.get_webview_window("danmaku") {
        window.set_ignore_cursor_events(new_pinned).map_err(|e| e.to_string())?;
    }

    let _ = app_handle.emit("pin-state-changed", new_pinned);

    Ok(new_pinned)
}

#[tauri::command]
pub fn set_danmaku_speed(app_handle: AppHandle, speed: f64) -> Result<(), String> {
    let _ = app_handle.emit("danmaku-speed", speed);
    Ok(())
}

fn create_danmaku_window(app_handle: &AppHandle) -> Result<(), String> {
    if app_handle.get_webview_window("danmaku").is_some() {
        if let Some(w) = app_handle.get_webview_window("danmaku") {
            let _ = w.show();
            let _ = w.set_focus();
        }
        return Ok(());
    }

    let url = WebviewUrl::App("index.html#/danmaku".into());
    let _window = WebviewWindowBuilder::new(app_handle, "danmaku", url)
        .title("弹幕显示")
        .inner_size(1200.0, 400.0)
        .position(100.0, 50.0)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .build()
        .map_err(|e| format!("Failed to create danmaku window: {}", e))?;

    Ok(())
}

fn create_control_bar_window(app_handle: &AppHandle) -> Result<(), String> {
    if app_handle.get_webview_window("control-bar").is_some() {
        if let Some(w) = app_handle.get_webview_window("control-bar") {
            let _ = w.show();
            let _ = w.set_focus();
        }
        return Ok(());
    }

    let url = WebviewUrl::App("index.html#/control-bar".into());
    let _window = WebviewWindowBuilder::new(app_handle, "control-bar", url)
        .title("弹幕控制")
        .inner_size(200.0, 36.0)
        .position(100.0, 450.0)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .build()
        .map_err(|e| format!("Failed to create control bar window: {}", e))?;

    Ok(())
}
