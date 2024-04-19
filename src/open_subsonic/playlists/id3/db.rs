use anyhow::Result;
use diesel::dsl::{count_distinct, max, sql, sum, AssumeNotNull};
use diesel::expression::SqlLiteral;
use diesel::{helper_types, sql_types, NullableExpressionMethods, Queryable, Selectable};
use futures::{stream, StreamExt, TryStreamExt};
use nghe_types::playlists::id3::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::sql::greatest;
use crate::open_subsonic::common::sql::greatest::HelperType as Greatest;
use crate::open_subsonic::id3::*;
use crate::DatabasePool;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct BasicPlaylistId3Db {
    pub id: Uuid,
    pub name: String,
    pub public: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(PartialEq))]
pub struct PlaylistId3Db {
    #[diesel(embed)]
    pub basic: BasicPlaylistId3Db,
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

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlaylistId3WithSongIdsDb {
    #[diesel(embed)]
    pub playlist: PlaylistId3Db,
    #[diesel(select_expression = sql::<sql_types::Array<sql_types::Uuid>>(
        "array_agg(playlists_songs.song_id order by playlists_songs.created_at asc) song_ids",
    ))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Array<sql_types::Uuid>>)]
    pub song_ids: Vec<Uuid>,
}

impl From<PlaylistId3Db> for PlaylistId3 {
    fn from(value: PlaylistId3Db) -> Self {
        Self {
            id: value.basic.id,
            name: value.basic.name,
            public: value.basic.public,
            created: value.basic.created_at,
            changed: value.updated_at,
            song_count: value.song_count as _,
            duration: value.duration as _,
        }
    }
}

impl From<BasicPlaylistId3Db> for PlaylistId3Db {
    fn from(value: BasicPlaylistId3Db) -> Self {
        Self { updated_at: value.created_at, basic: value, song_count: 0, duration: 0_f32 }
    }
}

impl From<BasicPlaylistId3Db> for PlaylistId3WithSongIdsDb {
    fn from(value: BasicPlaylistId3Db) -> Self {
        Self { playlist: value.into(), song_ids: vec![] }
    }
}

impl PlaylistId3WithSongIdsDb {
    pub async fn into(self, pool: &DatabasePool) -> Result<PlaylistId3WithSongs> {
        Ok(PlaylistId3WithSongs {
            playlist: self.playlist.into(),
            songs: stream::iter(get_songs(pool, &self.song_ids).await?)
                .then(|v| async move { v.into(pool).await })
                .try_collect()
                .await?,
        })
    }
}
