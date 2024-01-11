#![deny(clippy::all)]

pub mod config;
pub mod entity;
pub mod migrator;
pub mod open_subsonic;
pub mod state;
pub mod utils;

pub use migrator::Migrator;
pub use open_subsonic::{OSResult, OpenSubsonicError};
pub use state::ServerState;
