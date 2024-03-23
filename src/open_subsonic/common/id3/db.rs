use anyhow::Result;
use diesel::dsl::{count_distinct, max, sql, sum, AssumeNotNull};
use diesel::expression::SqlLiteral;
use diesel::{
    helper_types, sql_types, ExpressionMethods, NullableExpressionMethods, QueryDsl, Queryable,
    Selectable,
};
use diesel_async::RunQueryDsl;
use time::OffsetDateTime;
use uuid::Uuid;

use super::response::*;
use crate::models::*;
use crate::DatabasePool;

#[derive(Debug, Queryable)]
pub struct DateId3Db {
    pub year: Option<i16>,
    pub month: Option<i16>,
    pub day: Option<i16>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BasicArtistId3Db {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Queryable, Selectable)]
pub struct ArtistId3Db {
    #[diesel(embed)]
    pub basic: BasicArtistId3Db,
    #[diesel(select_expression = count_distinct(songs::album_id))]
    #[diesel(select_expression_type = count_distinct<songs::album_id>)]
    pub album_count: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = albums)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BasicAlbumId3Db {
    pub id: Uuid,
    pub name: String,
    #[diesel(select_expression = count_distinct(songs::id))]
    #[diesel(select_expression_type = count_distinct<songs::id>)]
    pub song_count: i64,
    #[diesel(select_expression = sum(songs::duration).assume_not_null())]
    #[diesel(select_expression_type = AssumeNotNull<helper_types::sum<songs::duration>>)]
    pub duration: f32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Queryable, Selectable)]
pub struct AlbumId3Db {
    #[diesel(embed)]
    pub basic: BasicAlbumId3Db,
    #[diesel(select_expression = sql(
        "array_agg(distinct(songs_album_artists.album_artist_id)) album_artist_ids",
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub artist_ids: Vec<Uuid>,
    #[diesel(select_expression = max(songs::year))]
    #[diesel(select_expression_type = helper_types::max<songs::year>)]
    pub year: Option<i16>,
    #[diesel(select_expression = (
        max(songs::release_year),
        max(songs::release_month),
        max(songs::release_day),
    ))]
    #[diesel(select_expression_type = (
        helper_types::max<songs::release_year>,
        helper_types::max<songs::release_month>,
        helper_types::max<songs::release_day>,
    ))]
    pub release_date: DateId3Db,
    #[diesel(select_expression = (
        max(songs::original_release_year),
        max(songs::original_release_month),
        max(songs::original_release_day),
    ))]
    #[diesel(select_expression_type = (
        helper_types::max<songs::original_release_year>,
        helper_types::max<songs::original_release_month>,
        helper_types::max<songs::original_release_day>,
    ))]
    pub original_release_date: DateId3Db,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BasicChildId3Db {
    pub id: Uuid,
    pub title: String,
    pub duration: f32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChildId3Db {
    #[diesel(embed)]
    pub basic: BasicChildId3Db,
    pub file_size: i64,
    pub format: String,
    pub bitrate: i32,
    pub album_id: Uuid,
    pub year: Option<i16>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    #[diesel(select_expression = sql("array_agg(songs_artists.artist_id) artist_ids"))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub artist_ids: Vec<Uuid>,
}

impl DateId3Db {
    pub fn into_res(self) -> DateId3 {
        DateId3 {
            year: self.year.map(|v| v as _),
            month: self.month.map(|v| v as _),
            day: self.day.map(|v| v as _),
        }
    }
}

impl BasicArtistId3Db {
    pub fn into_res(self) -> ArtistId3 {
        ArtistId3 { id: self.id, name: self.name, ..Default::default() }
    }
}

impl ArtistId3Db {
    pub fn into_res(self) -> ArtistId3 {
        ArtistId3 {
            id: self.basic.id,
            name: self.basic.name,
            album_count: Some(self.album_count as _),
        }
    }
}

impl BasicAlbumId3Db {
    pub fn into_res(self) -> AlbumId3 {
        AlbumId3 {
            id: self.id,
            name: self.name,
            song_count: self.song_count as _,
            duration: self.duration as _,
            created: self.created_at,
            ..Default::default()
        }
    }
}

impl AlbumId3Db {
    pub async fn into_res(self, pool: &DatabasePool) -> Result<AlbumId3> {
        let artists = artists::table
            .select((artists::id, artists::name))
            .filter(artists::id.eq_any(self.artist_ids))
            .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
            .await?
            .into_iter()
            .map(BasicArtistId3Db::into_res)
            .collect();

        Ok(AlbumId3 {
            id: self.basic.id,
            name: self.basic.name,
            song_count: self.basic.song_count as _,
            duration: self.basic.duration as _,
            created: self.basic.created_at,
            year: self.year.map(|v| v as _),
            artists,
            release_date: self.release_date.into_res(),
            original_release_date: self.original_release_date.into_res(),
        })
    }
}

impl BasicChildId3Db {
    pub fn into_res(self) -> ChildId3 {
        ChildId3 {
            id: self.id,
            is_dir: false,
            title: self.title,
            duration: self.duration as _,
            created: self.created_at,
            ..Default::default()
        }
    }
}

impl ChildId3Db {
    pub async fn into_res(self, pool: &DatabasePool) -> Result<ChildId3> {
        let artists = artists::table
            .select((artists::id, artists::name))
            .filter(artists::id.eq_any(self.artist_ids))
            .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
            .await?
            .into_iter()
            .map(BasicArtistId3Db::into_res)
            .collect();

        Ok(ChildId3 {
            id: self.basic.id,
            is_dir: false,
            title: self.basic.title,
            duration: self.basic.duration as _,
            created: self.basic.created_at,
            size: Some(self.file_size as _),
            content_type: Some(
                mime_guess::from_ext(&self.format).first_or_octet_stream().essence_str().to_owned(),
            ),
            suffix: Some(self.format),
            bit_rate: Some(self.bitrate as _),
            album_id: Some(self.album_id),
            year: self.year.map(|v| v as _),
            track: self.track_number.map(|v| v as _),
            disc_number: self.disc_number.map(|v| v as _),
            artists,
        })
    }
}
