use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{add_permission_filter, add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::query::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::permission::check_permission;
use crate::{Database, DatabasePool};

#[add_validate]
#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
pub struct Search3Params {
    artist_count: Option<i64>,
    artist_offset: Option<i64>,
    album_count: Option<i64>,
    album_offset: Option<i64>,
    song_count: Option<i64>,
    song_offset: Option<i64>,
    #[serde(rename = "musicFolderId")]
    music_folder_ids: Option<Vec<Uuid>>,
}

struct SearchOffsetCount {
    artist_count: i64,
    artist_offset: i64,
    album_count: i64,
    album_offset: i64,
    song_count: i64,
    song_offset: i64,
}

#[derive(Serialize)]
pub struct Search3Result {
    #[serde(rename = "artist", skip_serializing_if = "Vec::is_empty")]
    artists: Vec<ArtistId3>,
    #[serde(rename = "album", skip_serializing_if = "Vec::is_empty")]
    albums: Vec<AlbumId3>,
    #[serde(rename = "song", skip_serializing_if = "Vec::is_empty")]
    songs: Vec<SongId3>,
}

#[wrap_subsonic_response]
pub struct Search3Body {
    search_result_3: Search3Result,
}

async fn syncing(
    pool: &DatabasePool,
    user_id: Uuid,
    music_folder_ids: &Option<Vec<Uuid>>,
    SearchOffsetCount {
        artist_count,
        artist_offset,
        album_count,
        album_offset,
        song_count,
        song_offset,
    }: SearchOffsetCount,
) -> Result<Search3Result> {
    let artists = #[add_permission_filter]
    get_basic_artist_id3_db()
        .order(artists::name.asc())
        .limit(artist_count)
        .offset(artist_offset)
        .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
        .await?;

    let albums = #[add_permission_filter]
    get_album_id3_db()
        .order(albums::name.asc())
        .limit(album_count)
        .offset(album_offset)
        .get_results::<AlbumId3Db>(&mut pool.get().await?)
        .await?;

    let songs = #[add_permission_filter]
    get_song_id3_db()
        .order(songs::title.asc())
        .limit(song_count)
        .offset(song_offset)
        .get_results::<SongId3Db>(&mut pool.get().await?)
        .await?;

    Ok(Search3Result {
        artists: artists.into_iter().map(|v| v.into_res()).collect(),
        albums: stream::iter(albums)
            .then(|v| async move { v.into_res(pool).await })
            .try_collect()
            .await?,
        songs: stream::iter(songs)
            .then(|v| async move { v.into_res(pool).await })
            .try_collect()
            .await?,
    })
}

pub async fn search3_handler(
    State(database): State<Database>,
    req: Search3Request,
) -> Search3JsonResponse {
    check_permission(&database.pool, req.user_id, &req.params.music_folder_ids).await?;

    let search_offset_count = req.params.to_offset_count();

    let search_result =
        syncing(&database.pool, req.user_id, &req.params.music_folder_ids, search_offset_count)
            .await?;

    Search3Body { search_result_3: search_result }.into()
}

impl Search3Params {
    fn to_offset_count(&self) -> SearchOffsetCount {
        let Self {
            artist_count,
            artist_offset,
            album_count,
            album_offset,
            song_count,
            song_offset,
            ..
        } = self;
        SearchOffsetCount {
            artist_count: artist_count.unwrap_or(20),
            artist_offset: artist_offset.unwrap_or(0),
            album_count: album_count.unwrap_or(20),
            album_offset: album_offset.unwrap_or(0),
            song_count: song_count.unwrap_or(20),
            song_offset: song_offset.unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_syncing() {
        let n_song = 10;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        syncing(infra.pool(), infra.user_id(0), &None, Search3Params::default().to_offset_count())
            .await
            .unwrap();
    }
}
