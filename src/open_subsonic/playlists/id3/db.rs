use diesel::dsl::{count_distinct, max, sum, AssumeNotNull};
use diesel::{helper_types, NullableExpressionMethods, Queryable, Selectable};
use nghe_types::playlists::id3::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::sql::greatest;
use crate::open_subsonic::common::sql::greatest::HelperType as Greatest;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlaylistId3Db {
    pub id: Uuid,
    pub name: String,
    pub public: bool,
    pub created_at: OffsetDateTime,
    #[diesel(select_expression =
        greatest(max(playlists_songs::created_at).assume_not_null(), playlists::updated_at)
    )]
    #[diesel(select_expression_type =
        Greatest<
            AssumeNotNull<helper_types::max<playlists_songs::created_at>>,
            playlists::updated_at
        >
    )]
    pub updated_at: OffsetDateTime,
    #[diesel(select_expression = count_distinct(songs::id))]
    #[diesel(select_expression_type = count_distinct<songs::id>)]
    pub song_count: i64,
    #[diesel(select_expression = sum(songs::duration).assume_not_null())]
    #[diesel(select_expression_type = AssumeNotNull<helper_types::sum<songs::duration>>)]
    pub duration: f32,
}

impl From<PlaylistId3Db> for PlaylistId3 {
    fn from(value: PlaylistId3Db) -> Self {
        Self {
            id: value.id,
            name: value.name,
            public: value.public,
            created: value.created_at,
            changed: value.updated_at,
            song_count: value.song_count as _,
            duration: value.duration as _,
        }
    }
}
