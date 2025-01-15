use std::borrow::Cow;

use diesel::prelude::*;
use o2o::o2o;
use uuid::Uuid;

pub use crate::schema::users::{self, *};

#[derive(Debug, Clone, Copy, Queryable, Selectable, Insertable, o2o)]
#[diesel(table_name = users, check_for_backend(super::Type))]
#[map_owned(nghe_api::user::Role)]
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
#[diesel(table_name = users, check_for_backend(super::Type))]
pub struct Authenticated {
    pub id: Uuid,
    #[diesel(embed)]
    pub role: Role,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users, check_for_backend(super::Type))]
pub struct UsernameAuthentication<'a> {
    #[diesel(embed)]
    pub authenticated: Authenticated,
    pub password: Cow<'a, [u8]>,
}

#[derive(Debug, Queryable, Selectable, Insertable, o2o)]
#[diesel(table_name = users, check_for_backend(super::Type))]
#[owned_into(nghe_api::user::get::Response)]
pub struct Info<'a> {
    #[into(~.into_owned())]
    pub username: Cow<'a, str>,
    #[into(~.into_owned())]
    pub email: Cow<'a, str>,
    #[diesel(embed)]
    #[into(~.into())]
    pub role: Role,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = users, check_for_backend(super::Type))]
pub struct Data<'a> {
    #[diesel(embed)]
    pub info: Info<'a>,
    pub password: Cow<'a, [u8]>,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users, check_for_backend(super::Type))]
pub struct User<'a> {
    pub id: Uuid,
    #[diesel(embed)]
    pub data: Data<'a>,
}
