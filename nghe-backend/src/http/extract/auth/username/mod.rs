use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

pub trait UsernameAuthentication {
    fn username(&self) -> &str;
    fn authenticated(&self, password: impl AsRef<[u8]>) -> bool;
}

impl<A: UsernameAuthentication> super::Authentication for A {
    async fn authenticate(&self, database: &Database) -> Result<users::Authenticated, Error> {
        let user = users::table
            .filter(users::username.eq(self.username()))
            .select(users::UsernameAuthentication::as_select())
            .first(&mut database.get().await?)
            .await
            .map_err(|_| error::Kind::WrongUsernameOrPassword)?;
        if !self.authenticated(database.decrypt(&user.password)?) {
            error::Kind::WrongUsernameOrPassword.into()
        } else {
            Ok(user.authenticated)
        }
    }
}
