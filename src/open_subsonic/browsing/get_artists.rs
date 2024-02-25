use crate::{
    config::ArtistIndexConfig, models::*,
    open_subsonic::common::music_folder::check_user_music_folder_ids, Database, DatabasePool,
    OSResult, OpenSubsonicError,
};

use axum::extract::State;
use diesel::{
    dsl::count_distinct, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, Queryable,
};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct GetArtistsParams {
    music_folder_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IndexedArtist {
    id: Uuid,
    name: String,
    album_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    name: String,
    artists: Vec<IndexedArtist>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Indices {
    ignored_articles: String,
    index: Vec<Index>,
}

#[wrap_subsonic_response]
pub struct GetArtistsBody {
    artists: Indices,
}

async fn get_indexed_artists(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
) -> OSResult<Vec<(String, IndexedArtist)>> {
    Ok(artists::table
        .left_join(songs_album_artists::table)
        .left_join(songs_artists::table)
        .inner_join(
            songs::table.on(songs::id
                .eq(songs_album_artists::song_id)
                .or(songs::id.eq(songs_artists::song_id))),
        )
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .group_by(artists::id)
        .having(count_distinct(songs::album_id).gt(0))
        .select((
            artists::index,
            (artists::id, artists::name, count_distinct(songs::album_id)),
        ))
        .get_results::<(String, IndexedArtist)>(&mut pool.get().await?)
        .await?)
}

pub async fn get_artists_handler(
    State(database): State<Database>,
    req: GetArtistsRequest,
) -> OSResult<GetArtistsResponse> {
    let music_folder_ids = check_user_music_folder_ids(
        &database.pool,
        &req.user.id,
        req.params.music_folder_id.map(|m| vec![m].into()),
    )
    .await?;

    let ignored_articles = configs::table
        .select(configs::text)
        .filter(configs::key.eq(ArtistIndexConfig::IGNORED_ARTICLES_CONFIG_KEY))
        .first::<Option<String>>(&mut database.pool.get().await?)
        .await?
        .ok_or(OpenSubsonicError::NotFound {
            message: Some("ignored articles not found".into()),
        })?;

    let index = get_indexed_artists(&database.pool, &music_folder_ids)
        .await?
        .into_iter()
        .into_group_map()
        .into_iter()
        .map(|(k, v)| Index {
            name: k,
            artists: v,
        })
        .collect_vec();

    Ok(GetArtistsBody {
        artists: Indices {
            ignored_articles,
            index,
        },
    }
    .into())
}
