use anyhow::Result;
use diesel::dsl::{count_distinct, sql, sum, AssumeNotNull};
use diesel::expression::SqlLiteral;
use diesel::{
    helper_types, sql_types, ExpressionMethods, NullableExpressionMethods, QueryDsl, Queryable,
    Selectable, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use time::OffsetDateTime;
use uuid::Uuid;

use super::super::id::{MediaType, MediaTypedId};
use super::response::*;
use crate::models::*;
use crate::DatabasePool;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BasicArtistId3Db {
    pub id: Uuid,
    #[diesel(embed)]
    pub no_id: artists::ArtistNoId,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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
    #[diesel(embed)]
    pub no_id: albums::AlbumNoId,
    #[diesel(select_expression = count_distinct(songs::id))]
    #[diesel(select_expression_type = count_distinct<songs::id>)]
    pub song_count: i64,
    #[diesel(select_expression = sum(songs::duration).assume_not_null())]
    #[diesel(select_expression_type = AssumeNotNull<helper_types::sum<songs::duration>>)]
    pub duration: f32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = albums)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AlbumId3Db {
    #[diesel(embed)]
    pub basic: BasicAlbumId3Db,
    #[diesel(select_expression = sql(
        "array_agg(distinct(songs_album_artists.album_artist_id)) album_artist_ids",
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub artist_ids: Vec<Uuid>,
    #[diesel(embed)]
    pub genres: GenresId3Db,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BasicSongId3Db {
    pub id: Uuid,
    pub title: String,
    pub duration: f32,
    pub created_at: OffsetDateTime,
    pub file_size: i64,
    pub format: String,
    pub bitrate: i32,
    pub album_id: Uuid,
    pub year: Option<i16>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub cover_art_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SongId3Db {
    #[diesel(embed)]
    pub basic: BasicSongId3Db,
    #[diesel(select_expression = sql("array_agg(distinct(songs_artists.artist_id)) artist_ids"))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub artist_ids: Vec<Uuid>,
    #[diesel(embed)]
    pub genres: GenresId3Db,
}

pub type BasicGenreId3Db = genres::Genre;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = genres)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GenreId3Db {
    #[diesel(embed)]
    pub value: BasicGenreId3Db,
    #[diesel(select_expression = count_distinct(songs::album_id))]
    #[diesel(select_expression_type = count_distinct<songs::album_id>)]
    pub album_count: i64,
    #[diesel(select_expression = count_distinct(songs::id))]
    #[diesel(select_expression_type = count_distinct<songs::id>)]
    pub song_count: i64,
}

#[derive(Debug, Queryable, Selectable)]
pub struct GenresId3Db {
  #[diesel(select_expression = sql("array_agg(genres.value) genre_values"))]
  #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Nullable<sql_types::Text>>>)]
  pub genres: Vec<Option<String>>,
}

impl BasicArtistId3Db {
    pub fn into_res(self) -> ArtistId3 {
        ArtistId3::new(self.id, self.no_id.name.into_owned())
    }
}

impl ArtistId3Db {
    pub fn into_res(self) -> ArtistId3 {
        ArtistId3 {
            id: self.basic.id,
            name: self.basic.no_id.name.into_owned(),
            album_count: Some(self.album_count as _),
        }
    }
}

impl BasicAlbumId3Db {
    pub fn into_res(self) -> AlbumId3 {
        AlbumId3::new(
            self.id,
            self.no_id.name.into_owned(),
            self.no_id.date.year.map(|v| v as _),
            self.no_id.release_date.into(),
            self.no_id.original_release_date.into(),
            self.song_count as _,
            self.duration as _,
            self.created_at,
            MediaTypedId { t: Some(MediaType::Album), id: self.id },
        )
    }
}

impl AlbumId3Db {
    pub async fn into_res(self, pool: &DatabasePool) -> Result<AlbumId3> {
        let artists = artists::table
            .filter(artists::id.eq_any(self.artist_ids))
            .select(BasicArtistId3Db::as_select())
            .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
            .await?
            .into_iter()
            .map(BasicArtistId3Db::into_res)
            .collect();

        Ok(AlbumId3 {
            id: self.basic.id,
            name: self.basic.no_id.name.into_owned(),
            year: self.basic.no_id.date.year.map(|v| v as _),
            release_date: self.basic.no_id.release_date.into(),
            original_release_date: self.basic.no_id.original_release_date.into(),
            song_count: self.basic.song_count as _,
            duration: self.basic.duration as _,
            created: self.basic.created_at,
            cover_art: MediaTypedId { t: Some(MediaType::Album), id: self.basic.id },
            artists,
            genres: self.genres.into_res(),
        })
    }
}

impl BasicSongId3Db {
    pub fn into_res(self) -> SongId3 {
        SongId3::new(
            self.id,
            self.title,
            self.duration as _,
            self.created_at,
            self.file_size as _,
            self.format,
            self.bitrate as _,
            self.album_id,
            self.year.map(|v| v as _),
            self.track_number.map(|v| v as _),
            self.disc_number.map(|v| v as _),
            self.cover_art_id.map(|v| MediaTypedId { t: Some(MediaType::Song), id: v }),
        )
    }
}

impl SongId3Db {
    pub async fn into_res(self, pool: &DatabasePool) -> Result<SongId3> {
        let artists = artists::table
            .filter(artists::id.eq_any(self.artist_ids))
            .select(BasicArtistId3Db::as_select())
            .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
            .await?
            .into_iter()
            .map(BasicArtistId3Db::into_res)
            .collect();

        Ok(SongId3 {
            id: self.basic.id,
            title: self.basic.title,
            duration: self.basic.duration as _,
            created: self.basic.created_at,
            size: self.basic.file_size as _,
            bit_rate: self.basic.bitrate as _,
            album_id: self.basic.album_id,
            year: self.basic.year.map(|v| v as _),
            track: self.basic.track_number.map(|v| v as _),
            disc_number: self.basic.disc_number.map(|v| v as _),
            cover_art: self
                .basic
                .cover_art_id
                .map(|v| MediaTypedId { t: Some(MediaType::Song), id: v }),
            content_type: Some(
                mime_guess::from_ext(&self.basic.format)
                    .first_or_octet_stream()
                    .essence_str()
                    .to_owned(),
            ),
            suffix: self.basic.format,
            artists,
            genres: self.genres.into_res(),
        })
    }
}

impl GenreId3Db {
    pub fn into_res(self) -> GenreId3 {
        GenreId3 {
            value: self.value.value.into_owned(),
            song_count: self.song_count as _,
            album_count: self.album_count as _,
        }
    }
}

impl GenresId3Db {
  pub fn into_res(self) -> Vec<NameId3> {
    self.genres.into_iter().filter_map(|g| g.map(|g| NameId3 {name: g})).collect()
  }
}