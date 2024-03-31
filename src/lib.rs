#![deny(clippy::all)]
// TODO: reuse type when diesel 2.2.0 is released
#![allow(clippy::type_complexity)]
#![feature(ascii_char)]
#![feature(const_option, const_option_ext)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(try_blocks)]

use mimalloc::MiMalloc;

pub mod config;
pub mod database;
pub mod migration;
pub mod models;
pub mod open_subsonic;
pub mod schema;
pub mod utils;

pub use database::Database;
pub use open_subsonic::{OSError, ServerError};

pub type DatabaseType = diesel::pg::Pg;
pub type DatabasePool =
    diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
