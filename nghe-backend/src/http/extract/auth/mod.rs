mod api_key;
pub mod form;
pub mod header;
mod username;
pub use form::Form;
pub use header::Header;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

pub trait Authentication: Sized {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error>;
}
