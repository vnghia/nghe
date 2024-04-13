use std::borrow::Cow;

use diesel::prelude::*;
pub use lyrics::*;
use uuid::Uuid;

pub use crate::schema::lyrics;

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = lyrics)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateLyric<'a> {
    pub line_values: Vec<Option<Cow<'a, str>>>,
    pub line_starts: Option<Vec<Option<i32>>>,
    pub lyric_hash: i64,
    pub lyric_size: i32,
}

#[derive(Insertable, Identifiable)]
#[diesel(table_name = lyrics)]
#[diesel(primary_key(song_id, description, language, external))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LyricKey<'a> {
    pub song_id: Uuid,
    pub description: Cow<'a, str>,
    pub language: Cow<'a, str>,
    pub external: bool,
}

#[derive(Insertable)]
#[diesel(table_name = lyrics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewLyric<'a> {
    #[diesel(embed)]
    pub key: LyricKey<'a>,
    #[diesel(embed)]
    pub update: UpdateLyric<'a>,
}
