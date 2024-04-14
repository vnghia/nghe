use std::str::FromStr;

use anyhow::Result;
use diesel::dsl::{count_distinct, max, sql, sum, AssumeNotNull};
use diesel::expression::SqlLiteral;
use diesel::{
    helper_types, sql_types, ExpressionMethods, NullableExpressionMethods, QueryDsl, Queryable,
    Selectable, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use isolang::Language;
use nghe_types::id::{MediaType, MediaTypedId};
use nghe_types::id3::*;
use time::OffsetDateTime;
use uuid::Uuid;

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
    pub mbz_id: Option<Uuid>,
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
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BasicSongId3Db {
    pub id: Uuid,
    pub title: String,
    pub duration: f32,
    pub created_at: OffsetDateTime,
    pub file_size: i32,
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
    #[diesel(select_expression = max(albums::name).assume_not_null())]
    #[diesel(select_expression_type = AssumeNotNull<helper_types::max<albums::name>>)]
    pub album: String,
    #[diesel(select_expression = sql("array_agg(distinct(songs_artists.artist_id)) artist_ids"))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub artist_ids: Vec<Uuid>,
    #[diesel(embed)]
    pub genres: GenresId3Db,
    pub mbz_id: Option<Uuid>,
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
#[diesel(table_name = genres)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GenresId3Db {
    #[diesel(select_expression = sql("array_agg(distinct(genres.value)) genre_values"))]
    #[diesel(
      select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Nullable<sql_types::Text>>>
    )]
    pub genres: Vec<Option<String>>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = lyrics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LyricId3Db {
    pub language: String,
    pub line_values: Vec<Option<String>>,
    pub line_starts: Option<Vec<Option<i32>>>,
}

impl From<BasicArtistId3Db> for ArtistId3 {
    fn from(value: BasicArtistId3Db) -> Self {
        Self::new(value.id, value.no_id.name.into_owned())
    }
}

impl From<ArtistId3Db> for ArtistId3 {
    fn from(value: ArtistId3Db) -> Self {
        Self {
            id: value.basic.id,
            name: value.basic.no_id.name.into_owned(),
            album_count: Some(value.album_count as _),
            music_brainz_id: value.mbz_id,
        }
    }
}

impl From<BasicAlbumId3Db> for AlbumId3 {
    fn from(value: BasicAlbumId3Db) -> Self {
        Self::new(
            value.id,
            value.no_id.name.into_owned(),
            value.no_id.date.year.map(|v| v as _),
            value.no_id.release_date.into(),
            value.no_id.original_release_date.into(),
            value.song_count as _,
            value.duration as _,
            value.created_at,
            MediaTypedId { t: Some(MediaType::Album), id: value.id },
        )
    }
}

impl AlbumId3Db {
    pub async fn into(self, pool: &DatabasePool) -> Result<AlbumId3> {
        let artists = artists::table
            .filter(artists::id.eq_any(self.artist_ids))
            .select(BasicArtistId3Db::as_select())
            .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
            .await?
            .into_iter()
            .map(BasicArtistId3Db::into)
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
            genres: self.genres.into(),
            music_brainz_id: self.mbz_id,
        })
    }
}

impl From<BasicSongId3Db> for SongId3 {
    fn from(value: BasicSongId3Db) -> Self {
        Self::new(
            value.id,
            value.title,
            value.duration as _,
            value.created_at,
            value.file_size as _,
            value.format,
            value.bitrate as _,
            value.album_id,
            value.year.map(|v| v as _),
            value.track_number.map(|v| v as _),
            value.disc_number.map(|v| v as _),
            value.cover_art_id.map(|v| MediaTypedId { t: Some(MediaType::Song), id: v }),
        )
    }
}

impl SongId3Db {
    pub async fn into(self, pool: &DatabasePool) -> Result<SongId3> {
        let artists = artists::table
            .filter(artists::id.eq_any(self.artist_ids))
            .select(BasicArtistId3Db::as_select())
            .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
            .await?
            .into_iter()
            .map(BasicArtistId3Db::into)
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
            album: Some(self.album),
            content_type: Some(
                mime_guess::from_ext(&self.basic.format)
                    .first_or_octet_stream()
                    .essence_str()
                    .to_owned(),
            ),
            suffix: self.basic.format,
            artists,
            genres: self.genres.into(),
            music_brainz_id: self.mbz_id,
        })
    }
}

impl From<GenreId3Db> for GenreId3 {
    fn from(value: GenreId3Db) -> Self {
        Self {
            value: value.value.value.into_owned(),
            song_count: value.song_count as _,
            album_count: value.album_count as _,
        }
    }
}

impl From<GenresId3Db> for Vec<NameId3> {
    fn from(value: GenresId3Db) -> Self {
        value.genres.into_iter().filter_map(|g| g.map(|g| NameId3 { name: g })).collect()
    }
}

impl From<LyricId3Db> for LyricId3 {
    fn from(value: LyricId3Db) -> Self {
        let synced = value.line_starts.is_some();

        let line = if let Some(line_starts) = value.line_starts {
            line_starts
                .into_iter()
                .zip(value.line_values)
                .map(|(s, v)| LyricLineId3 { start: s.map(|s| s as _), value: v.unwrap() })
                .collect()
        } else {
            value
                .line_values
                .into_iter()
                .map(|v| LyricLineId3 { start: None, value: v.unwrap() })
                .collect()
        };

        Self {
            lang: Language::from_str(&value.language).expect("language inside database not found"),
            synced,
            line,
        }
    }
}
