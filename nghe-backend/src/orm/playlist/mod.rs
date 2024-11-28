pub mod permission;

use diesel::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::orm::playlists;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = false)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub comment: Option<String>,
    pub public: bool,
    #[diesel(column_name = created_at)]
    pub created: OffsetDateTime,
}
