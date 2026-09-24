mod api_key;
mod form;
mod header;
mod username;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

pub trait Authentication: Sized {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error>;
}
