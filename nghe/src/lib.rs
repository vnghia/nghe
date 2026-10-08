#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(coverage_attribute)]
#![feature(duration_constructors)]
#![feature(iterator_try_collect)]
#![feature(proc_macro_hygiene)]
#![feature(specialization)]
#![feature(stmt_expr_attributes)]
#![feature(str_as_str)]

#[coverage(off)]
pub mod command;
#[coverage(off)]
pub mod config;
mod database;
#[coverage(off)]
mod error;
mod file;
mod filesystem;
mod http;
mod integration;
pub mod migration;
mod orm;
mod route;
mod scan;
mod schema;
pub mod server;
mod sync;
mod time;

#[cfg(test)]
#[coverage(off)]
mod test;

use error::Error;
use rustfs_mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
