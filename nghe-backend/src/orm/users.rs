use std::borrow::Cow;

use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::users;

pub mod schema {
    pub use super::users::*;
}

pub use schema::table;

#[derive(Debug, Clone, Copy, Queryable, Selectable, Insertable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct Role {
    #[diesel(column_name = admin_role)]
    pub admin: bool,
    #[diesel(column_name = stream_role)]
    pub stream: bool,
    #[diesel(column_name = download_role)]
    pub download: bool,
    #[diesel(column_name = share_role)]
    pub share: bool,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct Auth<'a> {
    pub id: Uuid,
    pub password: Cow<'a, [u8]>,
    #[diesel(embed)]
    pub role: Role,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct Data<'a> {
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub password: Cow<'a, [u8]>,
    #[diesel(embed)]
    pub role: Role,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users, check_for_backend(crate::orm::Type))]
pub struct User<'a> {
    pub id: Uuid,
    #[diesel(embed)]
    pub data: Data<'a>,
}
