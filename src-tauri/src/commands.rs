use local_ip_address::local_ip;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub struct ServerState {
    pub running: AtomicBool,
    pub port: Mutex<u16>,
}

#[tauri::command]
pub async fn start_server(port: u16, app_handle: AppHandle) -> Result<String, String> {
    let state = app_handle.state::<ServerState>();

    if state.running.load(Ordering::SeqCst) {
        return Err("Server is already running".into());
    }

    crate::server::start_server(port, app_handle.clone()).await?;

    state.running.store(true, Ordering::SeqCst);
    *state.port.lock().map_err(|e| e.to_string())? = port;

    let ip = local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".into());
    let address = format!("http://{}:{}", ip, port);

    let _ = app_handle.emit("server-started", &address);

    Ok(address)
}

#[tauri::command]
pub fn stop_server(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<ServerState>();

    if !state.running.load(Ordering::SeqCst) {
        return Err("Server is not running".into());
    }

    state.running.store(false, Ordering::SeqCst);
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
