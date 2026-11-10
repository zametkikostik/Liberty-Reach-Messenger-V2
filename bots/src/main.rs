use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
struct Bot {
    id: String,
    username: String,
    name: String,
    created_at: String,
}

#[derive(Clone, Default)]
struct AppState {
    bots: Arc<RwLock<HashMap<String, Bot>>>,
}

#[derive(Deserialize)]
struct CreateBot {
    username: String,
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("liberty_bots=info")
        .init();
    let state = AppState::default();
    let app = Router::new()
        .route("/health", get(|| async { "bots-ok" }))
        .route("/api/v1/bots", post(create_bot).get(list_bots))
        .route("/api/v1/bots/:id", get(get_bot))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = std::env::var("BOTS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8081".into())
        .parse()?;
    tracing::info!("Liberty Bots on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn create_bot(State(s): State<AppState>, Json(b): Json<CreateBot>) -> Json<Bot> {
    let bot = Bot {
        id: Uuid::new_v4().to_string(),
        username: b.username,
        name: b.name,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    s.bots.write().await.insert(bot.id.clone(), bot.clone());
    Json(bot)
}

async fn list_bots(State(s): State<AppState>) -> Json<Vec<Bot>> {
    Json(s.bots.read().await.values().cloned().collect())
}

async fn get_bot(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Bot>, axum::http::StatusCode> {
    s.bots
        .read()
        .await
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(axum::http::StatusCode::NOT_FOUND)
}
