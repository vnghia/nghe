pub mod form;
pub mod header;
mod username;

use axum::extract::FromRef;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use form::Form;
pub use header::Header;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

pub trait Authentication {
    async fn authenticate(&self, database: &Database) -> Result<users::Authenticated, Error>;
}
