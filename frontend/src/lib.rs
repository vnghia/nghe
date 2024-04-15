#![deny(clippy::all)]
#![allow(non_snake_case)]
#![feature(associated_type_defaults)]
#![feature(let_chains)]
#![feature(try_blocks)]

mod components;
mod route;
mod state;
mod utils;

pub use route::Route;
