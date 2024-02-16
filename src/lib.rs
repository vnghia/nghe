#![deny(clippy::all)]

use mimalloc::MiMalloc;

pub mod config;
pub mod database;
pub mod migration;
pub mod models;
pub mod open_subsonic;
pub mod schema;
pub mod utils;

pub use database::Database;
pub use open_subsonic::{OSResult, OpenSubsonicError};

pub type DatabaseType = diesel::pg::Pg;
pub type DatabasePool =
    diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
