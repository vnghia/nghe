use std::borrow::Cow;

use diesel::prelude::*;
pub use playlists::*;

pub use crate::schema::playlists;

#[derive(Insertable)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPlaylist<'a> {
    pub name: Cow<'a, str>,
}

#[derive(AsChangeset)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdatePlaylist<'a> {
    pub name: Option<Cow<'a, str>>,
    pub comment: Option<Option<Cow<'a, str>>>,
    pub public: Option<bool>,
}
