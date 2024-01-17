use crate::{config::EncryptionKey, ServerState};
use axum::extract::State;
use sea_orm::DatabaseConnection;

pub fn setup_state(conn: &DatabaseConnection, key: EncryptionKey) -> State<ServerState> {
    State(ServerState {
        conn: conn.clone(),
        encryption_key: key,
    })
}
