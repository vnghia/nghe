#![feature(const_mut_refs)]
#![feature(try_blocks)]

mod app;
pub mod config;
mod error;
mod fs;
mod orm;
mod schema;

pub use app::{build, migration};
use error::Error;

#[cfg(test)]
mod test;
