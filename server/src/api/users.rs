use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Serialize;

use super::AppState;
use crate::auth::verify_token;

#[derive(Serialize)]
pub struct UserMe {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub verified: bool,
}

fn bearer(h: &HeaderMap) -> Option<String> {
    h.get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::to_string))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserMe>, StatusCode> {
    let token = bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = verify_token(&token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, i64)>(
        "SELECT id, username, display_name, bio, verified FROM users WHERE id = ?",
    )
    .bind(&claims.sub)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserMe {
        id: row.0,
        username: row.1,
        display_name: row.2,
        bio: row.3,
        verified: row.4 != 0,
    }))
}
