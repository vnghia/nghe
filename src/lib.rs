#![deny(clippy::all)]
#![allow(incomplete_features)]
// TODO: reuse type when diesel 2.2.0 is released
#![allow(clippy::type_complexity)]
#![feature(adt_const_params)]
#![feature(ascii_char)]
#![feature(const_option, const_option_ext)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]
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

#[cfg(test)]
pub mod params {
    pub use nghe_types::params::*;
}

#[cfg(test)]
pub mod response {
    pub use nghe_types::response::*;
}
