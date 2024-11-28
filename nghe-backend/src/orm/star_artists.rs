use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::star_artists::{self, *};

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = star_artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data {
    pub user_id: Uuid,
    pub artist_id: Uuid,
}
