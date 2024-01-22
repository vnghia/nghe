use crate::{
    config::EncryptionKey,
    state::{ArtistState, DatabaseState},
    DatabasePool, ServerState,
};

use axum::extract::State;

pub fn setup_state(pool: &DatabasePool, key: EncryptionKey) -> State<ServerState> {
    State(ServerState {
        database: DatabaseState {
            pool: pool.clone(),
            key,
        },
        artist: ArtistState::default(),
    })
}
