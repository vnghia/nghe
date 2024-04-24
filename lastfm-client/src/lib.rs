#![deny(clippy::all)]

pub mod artist;
mod client;
mod common;

pub use client::Client;
use common::*;
