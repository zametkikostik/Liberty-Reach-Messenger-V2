use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::auth::verify_token;

#[derive(Deserialize)]
pub struct SendMsg {
    /// Opaque payload — client sends E2EE ciphertext (lr1:...), server never decrypts
    pub body: String,
    pub reply_to: Option<String>,
    pub self_destruct_minutes: Option<i64>,
}

#[derive(Serialize)]
pub struct MessageDto {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub body: String,
    pub reply_to: Option<String>,
    pub self_destruct_at: Option<String>,
    pub created_at: String,
}

fn bearer(h: &HeaderMap) -> Option<String> {
    h.get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::to_string))
}

pub async fn send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(chat_id): Path<String>,
    Json(body): Json<SendMsg>,
) -> Result<Json<MessageDto>, StatusCode> {
    let token = bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = verify_token(&token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let self_destruct = body
        .self_destruct_minutes
        .map(|m| (now + chrono::Duration::minutes(m)).to_rfc3339());
    let created = now.to_rfc3339();

    // Store body as-is (ciphertext). No server-side decryption.
    sqlx::query(
        "INSERT INTO messages (id, chat_id, sender_id, body, reply_to, self_destruct_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(&chat_id)
    .bind(&claims.sub)
    .bind(&body.body)
    .bind(body.reply_to.as_deref())
    .bind(self_destruct.as_deref())
    .bind(&created)
    .execute(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dto = MessageDto {
        id: id.to_string(),
        chat_id: chat_id.clone(),
        sender_id: claims.sub.clone(),
        body: body.body.clone(),
        reply_to: body.reply_to.clone(),
        self_destruct_at: self_destruct.clone(),
        created_at: created.clone(),
    };

    // Fan-out to WS subscribers of this chat (still ciphertext only)
    let payload = serde_json::json!({
        "type": "message",
        "chat_id": chat_id,
        "id": dto.id,
        "sender_id": dto.sender_id,
        "body": dto.body,
        "created_at": dto.created_at,
    })
    .to_string();
    state.hub.publish(&chat_id, payload).await;

    Ok(Json(dto))
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(chat_id): Path<String>,
) -> Result<Json<Vec<MessageDto>>, StatusCode> {
    let token = bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let _ = verify_token(&token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
        ),
    >(
        "SELECT id, chat_id, sender_id, body, reply_to, self_destruct_at, created_at FROM messages WHERE chat_id = ? ORDER BY created_at ASC LIMIT 500",
    )
    .bind(&chat_id)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(
                |(id, chat_id, sender_id, body, reply_to, self_destruct_at, created_at)| {
                    MessageDto {
                        id,
                        chat_id,
                        sender_id,
                        body,
                        reply_to,
                        self_destruct_at,
                        created_at,
                    }
                },
            )
            .collect(),
    ))
}
