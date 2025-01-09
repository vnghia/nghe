use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::auth;

use crate::database::Database;
use crate::orm::{user_keys, users};
use crate::{Error, error};

impl super::Authentication for auth::ApiKey {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error> {
        users::table
            .inner_join(user_keys::table)
            .filter(user_keys::id.eq(self.api_key))
            .select(users::Authenticated::as_select())
            .first(&mut database.get().await?)
            .await
            .optional()?
            .ok_or_else(|| error::Kind::InvalidApiKey.into())
    }
}
