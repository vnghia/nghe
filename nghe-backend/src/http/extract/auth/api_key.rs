use nghe_api::auth;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

impl super::Authentication for auth::ApiKey {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error> {
        todo!()
    }
}
