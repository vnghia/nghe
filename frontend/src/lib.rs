#![deny(clippy::all)]
#![allow(non_snake_case)]
#![feature(let_chains)]
#![feature(try_blocks)]

mod components;
mod route;
mod state;

pub use route::Route;
