use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::auth;

use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

pub trait Authentication: Sized {
    fn username(&self) -> &str;
    fn authenticated(&self, password: impl AsRef<[u8]>) -> bool;
}

impl Authentication for auth::Username<'_, '_, '_, '_> {
    fn username(&self) -> &str {
        &self.username
    }

    fn authenticated(&self, password: impl AsRef<[u8]>) -> bool {
        let user_password = password.as_ref();
        match self.auth {
            auth::username::Auth::Token(ref auth) => {
                let password_token =
                    auth::username::Token::new(user_password, auth.salt.as_bytes());
                password_token == auth.token
            }
            auth::username::Auth::Password { ref password } => password.as_bytes() == user_password,
        }
    }
}

impl<A: Authentication> super::Authentication for A {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error> {
        let user = users::table
            .filter(users::username.eq(self.username()))
            .select(users::UsernameAuthentication::as_select())
            .first(&mut database.get().await?)
            .await
            .optional()?
            .ok_or_else(|| error::Kind::WrongUsernameOrPassword)?;
        if self.authenticated(database.decrypt(&user.password)?) {
            Ok(user.authenticated)
        } else {
            error::Kind::WrongUsernameOrPassword.into()
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    #![allow(unexpected_cfgs)]

    use std::borrow::Cow;

    use fake::Fake;
    use fake::faker::internet::en::{Password, UserAgent, Username};
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_authenticated(
        #[values(true, false)] with_token: bool,
        #[values(true, false)] ok: bool,
    ) {
        let password = Password(16..32).fake::<String>();
        let salt = Password(8..16).fake::<String>();
        let token = auth::username::Token::new(&password, &salt);

        let auth = if with_token {
            let salt = if ok { (&salt).into() } else { Password(8..16).fake::<String>().into() };
            auth::username::token::Auth { salt, token }.into()
        } else {
            let password: Cow<'_, str> =
                if ok { (&password).into() } else { Password(8..16).fake::<String>().into() };
            password.into()
        };

        let username = auth::Username {
            username: Username().fake::<String>().into(),
            client: UserAgent().fake::<String>().into(),
            auth,
        };
        assert_eq!(username.authenticated(password.as_bytes()), ok);
    }
}
