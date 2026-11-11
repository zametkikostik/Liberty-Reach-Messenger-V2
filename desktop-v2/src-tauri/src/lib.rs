//! Liberty Messenger V2 — Tauri 2 library entry (desktop + future mobile).
//!
//! Frontend is the shared React app under `frontend/`.
//! Native process never sees message plaintext — E2EE stays in the webview.

use tauri::Manager;

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "Liberty Messenger V2",
        "version": env!("CARGO_PKG_VERSION"),
        "e2ee": "client-side AES-256-GCM (lr1:)",
        "kyber": cfg!(feature = "kyber"),
        "p2p": "relay-via-wshub",
        "platform": std::env::consts::OS,
    })
}

/// Base URL of the Axum backend (override with LIBERTY_SERVER_URL).
#[tauri::command]
fn backend_url() -> String {
    std::env::var("LIBERTY_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8008".into())
}

/// Health probe from native side (optional; UI also hits /health via proxy).
#[tauri::command]
async fn backend_health(url: Option<String>) -> Result<serde_json::Value, String> {
    let base = url.unwrap_or_else(backend_url);
    let endpoint = format!("{}/health", base.trim_end_matches('/'));
    Ok(serde_json::json!({
        "configured": base,
        "endpoint": endpoint,
        "note": "Start server with: cd server && cargo run"
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![app_info, backend_url, backend_health])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("Liberty Messenger V2");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Liberty desktop");
}
