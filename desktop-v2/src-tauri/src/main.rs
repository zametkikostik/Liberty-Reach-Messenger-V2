// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Liberty Messenger V2 — Tauri 2 desktop entry.
//!
//! UI is the same React frontend (../frontend or bundled dist).
//! Native commands expose crypto helpers and open the local server URL.

use tauri::Manager;

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "Liberty Messenger V2",
        "version": env!("CARGO_PKG_VERSION"),
        "e2ee": "client-side AES-256-GCM (lr1:)",
        "kyber": cfg!(feature = "kyber"),
        "p2p": "relay-via-wshub",
    })
}

#[tauri::command]
fn open_backend_url() -> String {
    std::env::var("LIBERTY_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8008".into())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![app_info, open_backend_url])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("Liberty Messenger V2");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Liberty desktop");
}
