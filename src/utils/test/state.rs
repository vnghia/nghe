use crate::{config::EncryptionKey, DbPool, ServerState};
use axum::extract::State;

pub fn setup_state(pool: &DbPool, key: EncryptionKey) -> State<ServerState> {
    State(ServerState {
        pool: pool.clone(),
        encryption_key: key,
    })
}
