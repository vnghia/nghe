mod api_key;
pub mod form;
pub mod header;
mod username;
use axum::extract::FromRef;
pub use form::Form;
pub use header::Header;

use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

pub trait Authentication: Sized {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error>;

    async fn login<S, R>(&self, state: &S) -> Result<users::Authenticated, Error>
    where
        S: Send + Sync,
        Database: FromRef<S>,
        R: Authorization,
    {
        let database = Database::from_ref(state);
        let user = self.authenticated(&database).await?;
        if R::authorized(user.role) { Ok(user) } else { error::Kind::Forbidden.into() }
    }
}

pub trait Authorization {
    fn authorized(role: users::Role) -> bool;
}
