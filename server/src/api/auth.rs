use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::auth::{hash_password, issue_token, verify_password};

#[derive(Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResp {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterReq>,
) -> Result<Json<AuthResp>, StatusCode> {
    if body.username.len() < 3 || body.password.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let id = Uuid::new_v4();
    let hash = hash_password(&body.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, display_name, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(&body.username)
    .bind(&hash)
    .bind(body.display_name.as_deref())
    .bind(&now)
    .execute(&state.db.pool)
    .await
    .map_err(|_| StatusCode::CONFLICT)?;

    let token =
        issue_token(id, &body.username, &state.jwt_secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AuthResp {
        token,
        user_id: id.to_string(),
        username: body.username,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginReq>,
) -> Result<Json<AuthResp>, StatusCode> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT id, password_hash FROM users WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(&body.password, &row.1) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let uid = Uuid::parse_str(&row.0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token =
        issue_token(uid, &body.username, &state.jwt_secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AuthResp {
        token,
        user_id: row.0,
        username: body.username,
    }))
}
