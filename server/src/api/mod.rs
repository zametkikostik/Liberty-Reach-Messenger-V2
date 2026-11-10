pub mod auth;
pub mod chats;
pub mod messages;
pub mod users;

use crate::db::Db;
use crate::websocket::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub jwt_secret: String,
    pub hub: WsHub,
}
