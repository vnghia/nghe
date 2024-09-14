#![feature(adt_const_params)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(const_mut_refs)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(try_blocks)]

mod app;
pub mod config;
mod error;
mod filesystem;
mod media;
mod orm;
mod schema;

pub use app::{build, migration};
use error::Error;

#[cfg(test)]
mod test;
