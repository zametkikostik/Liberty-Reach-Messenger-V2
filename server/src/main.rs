mod api;
mod auth;
mod db;
mod websocket;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "liberty_server=info,tower_http=info".into()),
        )
        .init();

    let db = db::Db::connect().await?;
    db.migrate().await?;

    let state = api::AppState {
        db,
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "liberty-dev-secret-change-me-32chars!!".into()),
        hub: websocket::WsHub::new(),
    };

    let app = Router::new()
        .route(
            "/health",
            get(|| async {
                axum::Json(serde_json::json!({"ok":true,"service":"liberty-server","e2ee":"client-side"}))
            }),
        )
        .route("/api/v1/auth/register", post(api::auth::register))
        .route("/api/v1/auth/login", post(api::auth::login))
        .route("/api/v1/users/me", get(api::users::me))
        .route("/api/v1/chats", get(api::chats::list).post(api::chats::create))
        .route(
            "/api/v1/chats/:id/messages",
            get(api::messages::list).post(api::messages::send),
        )
        .route("/ws", get(websocket::ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr: SocketAddr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8008".into())
        .parse()?;
    tracing::info!("Liberty server on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
