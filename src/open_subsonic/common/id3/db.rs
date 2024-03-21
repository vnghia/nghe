use super::response::*;
use crate::{models::*, DatabasePool};

use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl, Queryable};
use diesel_async::RunQueryDsl;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Queryable)]
pub struct DateId3Db {
    pub year: Option<i16>,
    pub month: Option<i16>,
    pub day: Option<i16>,
}

#[derive(Debug, Queryable)]
pub struct BasicArtistId3Db {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Queryable)]
pub struct ArtistId3Db {
    pub basic: BasicArtistId3Db,
    pub album_count: i64,
}

#[derive(Debug, Queryable)]
pub struct BasicAlbumId3Db {
    pub id: Uuid,
    pub name: String,
    pub song_count: i64,
    pub duration: f32,
    pub created: OffsetDateTime,
}

#[derive(Debug, Queryable)]
pub struct AlbumId3Db {
    pub basic: BasicAlbumId3Db,
    pub artist_ids: Vec<Uuid>,
    pub year: Option<i16>,
    pub release_date: DateId3Db,
    pub original_release_date: DateId3Db,
}

#[derive(Debug, Queryable)]
pub struct BasicChildId3Db {
    pub id: Uuid,
    pub title: String,
    pub duration: f32,
    pub created: OffsetDateTime,
}

#[derive(Debug, Queryable)]
pub struct ChildId3Db {
    pub basic: BasicChildId3Db,
    pub size: i64,
    pub format: String,
    pub bit_rate: i32,
    pub album_id: Uuid,
    pub year: Option<i16>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
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
        ArtistId3 {
            id: self.id,
            name: self.name,
            ..Default::default()
        }
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
            created: self.created,
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
            created: self.basic.created,
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
            created: self.created,
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
            created: self.basic.created,
            size: Some(self.size as _),
            content_type: Some(
                mime_guess::from_ext(&self.format)
                    .first_or_octet_stream()
                    .essence_str()
                    .to_owned(),
            ),
            suffix: Some(self.format),
            bit_rate: Some(self.bit_rate as _),
            album_id: Some(self.album_id),
            year: self.year.map(|v| v as _),
            track: self.track_number.map(|v| v as _),
            disc_number: self.disc_number.map(|v| v as _),
            artists,
        })
    }
}
