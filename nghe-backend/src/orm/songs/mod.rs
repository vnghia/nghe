pub mod date;
pub mod position;

use crate::schema::songs;

pub mod schema {
    pub use super::songs::*;
}

pub use schema::table;
