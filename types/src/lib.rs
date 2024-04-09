#![deny(clippy::all)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(ascii_char)]
#![feature(const_option)]

pub mod bookmarks;
pub mod browsing;
pub mod common;
pub mod extension;
pub mod media_list;
pub mod media_retrieval;
pub mod scan;
pub mod searching;
pub mod system;
pub mod user;

pub use common::*;
