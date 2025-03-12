use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;
use o2o::o2o;
use uuid::Uuid;

pub use crate::schema::users::{self, *};

#[derive(Debug, Clone, Copy, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
#[map_owned(nghe_api::user::Role)]
#[cfg_attr(test, derive(Default, PartialEq, Eq))]
pub struct Role {
    pub admin: bool,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct Authenticated {
    pub id: Uuid,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct UsernameAuthentication<'a> {
    #[diesel(embed)]
    pub authenticated: Authenticated,
    pub password: Cow<'a, [u8]>,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct Info<'a> {
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    #[diesel(embed)]
    pub role: Role,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct Data<'a> {
    #[diesel(embed)]
    pub info: Info<'a>,
    pub password: Cow<'a, [u8]>,
}

#[derive(Debug, Queryable, Selectable, Identifiable, o2o)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
#[owned_into(nghe_api::user::get::Response)]
#[ghosts(
    username: {@.info.username.into_owned()},
    email: {@.info.email.into_owned()},
    role: {@.info.role.into()}
)]
pub struct User<'a> {
    pub id: Uuid,
    #[diesel(embed)]
    #[ghost]
    pub info: Info<'a>,
}

#[cfg(test)]
#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct Full<'a> {
    pub id: Uuid,
    #[diesel(embed)]
    pub data: Data<'a>,
}

mod check {
    use diesel_async::RunQueryDsl;

    use super::*;
    use crate::database::Database;
    use crate::{Error, error};

    impl Role {
        pub async fn query(database: &Database, user_id: Uuid) -> Result<Self, Error> {
            users::table
                .filter(users::id.eq(user_id))
                .select(Self::as_select())
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }

        pub async fn check_admin(database: &Database, user_id: Uuid) -> Result<(), Error> {
            let role = Self::query(database, user_id).await?;
            if role.admin { Ok(()) } else { error::Kind::Forbidden.into() }
        }
    }
}
