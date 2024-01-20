#![deny(clippy::all)]

use mimalloc::MiMalloc;

pub mod config;
pub mod migration;
pub mod models;
pub mod open_subsonic;
pub mod schema;
pub mod state;
pub mod utils;

pub use open_subsonic::{OSResult, OpenSubsonicError};
pub use state::ServerState;

pub type DbPool = diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
