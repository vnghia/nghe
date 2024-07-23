use anyhow::Result;
use diesel::dsl::{count, max, sql, sum, Nullable};
use diesel::expression::SqlLiteral;
use diesel::{helper_types, sql_types, NullableExpressionMethods, Queryable, Selectable};
use futures::{stream, StreamExt, TryStreamExt};
use nghe_types::playlists::id3::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::sql::greatest_tz;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::sql::coalesce_f32;
use crate::DatabasePool;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct BasicPlaylistId3Db {
    pub id: Uuid,
    pub name: String,
    pub comment: Option<String>,
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
        greatest_tz(max(playlists_songs::created_at.nullable()), playlists::updated_at)
    )]
    #[diesel(select_expression_type =
        greatest_tz<
            helper_types::max<Nullable<playlists_songs::created_at>>,
            playlists::updated_at
        >
    )]
    pub updated_at: OffsetDateTime,
    #[diesel(select_expression = count(songs::id.nullable()))]
    #[diesel(select_expression_type = count<Nullable<songs::id>>)]
    pub song_count: i64,
    #[diesel(select_expression = coalesce_f32(sum(songs::duration.nullable()), 0_f32))]
    #[diesel(select_expression_type =
        coalesce_f32<helper_types::sum<Nullable<songs::duration>>, f32>
    )]
    pub duration: f32,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlaylistId3WithSongIdsDb {
    #[diesel(embed)]
    pub playlist: PlaylistId3Db,
    #[diesel(select_expression = sql::<sql_types::Array<sql_types::Uuid>>(
        "coalesce(array_agg(playlists_songs.song_id order by playlists_songs.created_at asc) \
            filter (where playlists_songs.song_id is not null), '{}') song_ids",
    ))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Array<sql_types::Uuid>>)]
    pub song_ids: Vec<Uuid>,
}

impl From<PlaylistId3Db> for PlaylistId3 {
    fn from(value: PlaylistId3Db) -> Self {
        Self {
            id: value.basic.id,
            name: value.basic.name,
            comment: value.basic.comment,
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
