use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::auth::verify_token;

#[derive(Deserialize)]
pub struct CreateChat {
    pub chat_type: String,
    pub title: Option<String>,
    pub member_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct ChatDto {
    pub id: String,
    pub chat_type: String,
    pub title: Option<String>,
    pub created_at: String,
}

fn bearer(h: &HeaderMap) -> Option<String> {
    h.get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::to_string))
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateChat>,
) -> Result<Json<ChatDto>, StatusCode> {
    let token = bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = verify_token(&token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO chats (id, chat_type, title, created_at) VALUES (?, ?, ?, ?)")
        .bind(id.to_string())
        .bind(&body.chat_type)
        .bind(body.title.as_deref())
        .bind(&now)
        .execute(&state.db.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut members = body.member_ids;
    if !members.iter().any(|m| m == &claims.sub) {
        members.push(claims.sub);
    }
    for m in members {
        let _ = sqlx::query("INSERT OR IGNORE INTO chat_members (chat_id, user_id) VALUES (?, ?)")
            .bind(id.to_string())
            .bind(m)
            .execute(&state.db.pool)
            .await;
    }
    Ok(Json(ChatDto {
        id: id.to_string(),
        chat_type: body.chat_type,
        title: body.title,
        created_at: now,
    }))
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ChatDto>>, StatusCode> {
    let token = bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = verify_token(&token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        r#"SELECT c.id, c.chat_type, c.title, c.created_at
           FROM chats c JOIN chat_members m ON m.chat_id = c.id
           WHERE m.user_id = ? ORDER BY c.created_at DESC"#,
    )
    .bind(&claims.sub)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, chat_type, title, created_at)| ChatDto {
                id,
                chat_type,
                title,
                created_at,
            })
            .collect(),
    ))
}
