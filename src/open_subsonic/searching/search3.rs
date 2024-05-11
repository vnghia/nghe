use std::borrow::Cow;

use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use diesel_full_text_search::configuration::TsConfigurationByName;
use diesel_full_text_search::*;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{
    add_axum_response, add_common_validate, add_count_offset, add_permission_filter,
};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::check_permission;
use crate::{Database, DatabasePool};

const USIMPLE_TS_CONFIGURATION: TsConfigurationByName = TsConfigurationByName("usimple");

add_common_validate!(Search3Params);
add_axum_response!(Search3Body);

#[derive(Debug)]
struct SearchQueryParams<'a> {
    query: Cow<'a, str>,
    artist_count: u32,
    artist_offset: u32,
    album_count: u32,
    album_offset: u32,
    song_count: u32,
    song_offset: u32,
}

impl<'a> From<&'a Search3Params> for SearchQueryParams<'a> {
    fn from(value: &'a Search3Params) -> Self {
        Self {
            query: value.query.as_str().into(),
            artist_count: value.artist_count.unwrap_or(20),
            artist_offset: value.artist_offset.unwrap_or(0),
            album_count: value.album_count.unwrap_or(20),
            album_offset: value.album_offset.unwrap_or(0),
            song_count: value.song_count.unwrap_or(20),
            song_offset: value.song_offset.unwrap_or(0),
        }
    }
}

async fn sync(
    pool: &DatabasePool,
    user_id: Uuid,
    music_folder_ids: &Option<Vec<Uuid>>,
    SearchQueryParams {
        artist_count,
        artist_offset,
        album_count,
        album_offset,
        song_count,
        song_offset,
        ..
    }: SearchQueryParams<'_>,
) -> Result<Search3Result> {
    let mut artists = #[add_permission_filter]
    #[add_count_offset(artist)]
    get_album_artist_id3_db()
        .order(artists::name.asc())
        .get_results(&mut pool.get().await?)
        .await?;
    if !artists.is_empty() && artists.len() < artist_count as usize {
        // the number of artists with no album is relatively small, so we include it in the response
        // if the number of artists with album is smaller than the requested number.
        // if the number of artists with album is divisible by artist count, the artist with no
        // album will be ignored. In order to resolve this, we will need a continuation token.
        artists.extend({
            #[add_permission_filter]
            get_no_album_artist_id3_db()
                .order(artists::name.asc())
                .get_results(&mut pool.get().await?)
                .await?
                .into_iter()
                .map(ArtistId3Db::into)
        });
    };

    let albums = #[add_permission_filter]
    #[add_count_offset(album)]
    get_album_id3_db()
        .order(albums::name.asc())
        .get_results::<AlbumId3Db>(&mut pool.get().await?)
        .await?;

    let songs = #[add_permission_filter]
    #[add_count_offset(song)]
    get_song_id3_db()
        .order(songs::title.asc())
        .get_results::<SongId3Db>(&mut pool.get().await?)
        .await?;

    Ok(Search3Result {
        artists: artists.into_iter().map(ArtistAlbumCountId3Db::into).collect(),
        albums: stream::iter(albums)
            .then(|v| async move { v.into(pool).await })
            .try_collect()
            .await?,
        songs: stream::iter(songs)
            .then(|v| async move { v.into(pool).await })
            .try_collect()
            .await?,
    })
}

async fn full_text_search(
    pool: &DatabasePool,
    user_id: Uuid,
    music_folder_ids: &Option<Vec<Uuid>>,
    SearchQueryParams {
        query,
        artist_count,
        artist_offset,
        album_count,
        album_offset,
        song_count,
        song_offset,
    }: SearchQueryParams<'_>,
) -> Result<Search3Result> {
    let mut artists = #[add_permission_filter]
    #[add_count_offset(artist)]
    get_album_artist_id3_db()
        .filter(
            artists::ts
                .matches(websearch_to_tsquery_with_search_config(USIMPLE_TS_CONFIGURATION, &query)),
        )
        .order(
            ts_rank_cd(
                artists::ts,
                websearch_to_tsquery_with_search_config(USIMPLE_TS_CONFIGURATION, &query),
            )
            .desc(),
        )
        .get_results(&mut pool.get().await?)
        .await?;
    if !artists.is_empty() && artists.len() < artist_count as usize {
        // the number of artists with no album is relatively small, so we include it in the response
        // if the number of artists with album is smaller than the requested number.
        // if the number of artists with album is divisible by artist count, the artist with no
        // album will be ignored. In order to resolve this, we will need a continuation token.
        artists.extend({
            #[add_permission_filter]
            get_no_album_artist_id3_db()
                .filter(artists::ts.matches(websearch_to_tsquery_with_search_config(
                    USIMPLE_TS_CONFIGURATION,
                    &query,
                )))
                .order(
                    ts_rank_cd(
                        artists::ts,
                        websearch_to_tsquery_with_search_config(USIMPLE_TS_CONFIGURATION, &query),
                    )
                    .desc(),
                )
                .get_results(&mut pool.get().await?)
                .await?
                .into_iter()
                .map(ArtistId3Db::into)
        });
    }

    let albums = #[add_permission_filter]
    #[add_count_offset(album)]
    get_basic_album_id3_db()
        .filter(
            albums::ts
                .matches(websearch_to_tsquery_with_search_config(USIMPLE_TS_CONFIGURATION, &query)),
        )
        .order(
            ts_rank_cd(
                albums::ts,
                websearch_to_tsquery_with_search_config(USIMPLE_TS_CONFIGURATION, &query),
            )
            .desc(),
        )
        .get_results::<BasicAlbumId3Db>(&mut pool.get().await?)
        .await?;

    let songs = #[add_permission_filter]
    #[add_count_offset(song)]
    get_song_id3_db()
        .filter(
            songs::ts
                .matches(websearch_to_tsquery_with_search_config(USIMPLE_TS_CONFIGURATION, &query)),
        )
        .order(
            ts_rank_cd(
                songs::ts,
                websearch_to_tsquery_with_search_config(USIMPLE_TS_CONFIGURATION, &query),
            )
            .desc(),
        )
        .get_results::<SongId3Db>(&mut pool.get().await?)
        .await?;

    Ok(Search3Result {
        artists: artists.into_iter().map(ArtistAlbumCountId3Db::into).collect(),
        albums: albums.into_iter().map(BasicAlbumId3Db::into).collect(),
        songs: stream::iter(songs)
            .then(|v| async move { v.into(pool).await })
            .try_collect()
            .await?,
    })
}

pub async fn search3_handler(
    State(database): State<Database>,
    req: Search3Request,
) -> Search3JsonResponse {
    check_permission(&database.pool, req.user_id, &req.params.music_folder_ids).await?;

    let search_query: SearchQueryParams = (&req.params).into();

    let search_result = if search_query.query.is_empty() || search_query.query == "\"\"" {
        sync(&database.pool, req.user_id, &req.params.music_folder_ids, search_query).await
    } else {
        full_text_search(&database.pool, req.user_id, &req.params.music_folder_ids, search_query)
            .await
    }?;

    Ok(axum::Json(Search3Body { search_result3: search_result }.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::Infra;

    fn default_search3_params() -> Search3Params {
        Search3Params {
            query: Default::default(),
            artist_count: None,
            artist_offset: None,
            album_count: None,
            album_offset: None,
            song_count: None,
            song_offset: None,
            music_folder_ids: None,
        }
    }

    #[tokio::test]
    async fn test_sync() {
        let n_song = 10;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, n_song).await.scan(.., None).await;
        sync(infra.pool(), infra.user_id(0), &None, (&default_search3_params()).into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_full_text_search() {
        let n_song = 10;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, n_song).await.scan(.., None).await;
        full_text_search(
            infra.pool(),
            infra.user_id(0),
            &None,
            (&Search3Params { query: "search".into(), ..default_search3_params() }).into(),
        )
        .await
        .unwrap();
    }
}
