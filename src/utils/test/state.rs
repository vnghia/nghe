use super::db::TemporaryDatabase;
use crate::{
    state::{ArtistState, DatabaseState},
    ServerState,
};

use axum::extract::State;

pub fn setup_state(db: &TemporaryDatabase) -> State<ServerState> {
    State(ServerState {
        database: DatabaseState {
            pool: db.get_pool().to_owned(),
            key: db.get_key().to_owned(),
        },
        artist: ArtistState::default(),
    })
}
