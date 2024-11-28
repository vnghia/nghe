use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::star_albums::{self, *};

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = star_albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data {
    pub user_id: Uuid,
    pub album_id: Uuid,
}
