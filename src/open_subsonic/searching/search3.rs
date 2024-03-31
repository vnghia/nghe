use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::query::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::common::music_folder::check_user_music_folder_ids;
use crate::{Database, DatabasePool};

#[add_validate]
#[derive(Debug)]
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
    music_folder_ids: &[Uuid],
    SearchOffsetCount {
        artist_count,
        artist_offset,
        album_count,
        album_offset,
        song_count,
        song_offset,
    }: SearchOffsetCount,
) -> Result<Search3Result> {
    let artists = get_basic_artist_id3_db()
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .order(artists::name.asc())
        .limit(artist_count)
        .offset(artist_offset)
        .get_results::<BasicArtistId3Db>(&mut pool.get().await?)
        .await?;

    let albums = get_album_id3_db()
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .order(albums::name.asc())
        .limit(album_count)
        .offset(album_offset)
        .get_results::<AlbumId3Db>(&mut pool.get().await?)
        .await?;

    let songs = get_song_id3_db()
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
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
    let Search3Params {
        artist_count,
        artist_offset,
        album_count,
        album_offset,
        song_count,
        song_offset,
        music_folder_ids,
    } = req.params;
    let search_offset_count = SearchOffsetCount {
        artist_count: artist_count.unwrap_or(20),
        artist_offset: artist_offset.unwrap_or(0),
        album_count: album_count.unwrap_or(20),
        album_offset: album_offset.unwrap_or(0),
        song_count: song_count.unwrap_or(20),
        song_offset: song_offset.unwrap_or(0),
    };

    let music_folder_ids = check_user_music_folder_ids(
        &database.pool,
        &req.user_id,
        music_folder_ids.map(|v| v.into()),
    )
    .await?;

    let search_result = syncing(&database.pool, &music_folder_ids, search_offset_count).await?;

    Search3Body { search_result_3: search_result }.into()
}
