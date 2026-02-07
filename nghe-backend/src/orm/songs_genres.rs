use diesel::prelude::*;
use uuid::Uuid;

pub use crate::schema::songs_genres::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs_genres, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data {
    pub song_id: Uuid,
    pub genre_id: Uuid,
}
