use crate::{
    models::*,
    open_subsonic::common::{
        id3::{db::*, response::*},
        music_folder::check_user_music_folder_ids,
    },
    Database, DatabasePool, OSError,
};

use anyhow::Result;
use axum::extract::State;
use diesel::{
    dsl::{count_distinct, sql},
    sql_types, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use super::get_album::get_basic_songs;

#[add_validate]
#[derive(Debug)]
pub struct GetMusicDirectoryParams {
    id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicArtistId3WithSongs {
    #[serde(flatten)]
    artist: ArtistId3,
    #[serde(rename = "child")]
    children: Vec<ChildId3>,
}

#[wrap_subsonic_response]
pub struct GetMusicDirectoryBody {
    directory: BasicArtistId3WithSongs,
}

async fn get_basic_artist_and_song_ids(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    artist_id: &Uuid,
) -> Result<(BasicArtistId3Db, Vec<Uuid>)> {
    artists::table
        .left_join(songs_album_artists::table)
        .left_join(songs_artists::table)
        .inner_join(
            songs::table.on(songs::id
                .eq(songs_album_artists::song_id)
                .or(songs::id.eq(songs_artists::song_id))),
        )
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(artists::id.eq(artist_id))
        .group_by(artists::id)
        .having(count_distinct(songs::id).gt(0))
        .select((
            BasicArtistId3Db::as_select(),
            sql::<sql_types::Array<sql_types::Uuid>>("array_agg(distinct(songs.id)) song_ids"),
        ))
        .first::<(BasicArtistId3Db, Vec<Uuid>)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Music directory".into()).into())
}

async fn get_music_directory(
    pool: &DatabasePool,
    user_id: Uuid,
    artist_id: Uuid,
) -> Result<BasicArtistId3WithSongs> {
    let music_folder_ids = check_user_music_folder_ids(pool, &user_id, None).await?;

    let (artist, song_ids) =
        get_basic_artist_and_song_ids(pool, &music_folder_ids, &artist_id).await?;
    let basic_songs = get_basic_songs(pool, &music_folder_ids, &song_ids).await?;

    Ok(BasicArtistId3WithSongs {
        artist: artist.into_res(),
        children: basic_songs.into_iter().map(|v| v.into_res()).collect(),
    })
}

pub async fn get_music_directory_handler(
    State(database): State<Database>,
    req: GetMusicDirectoryRequest,
) -> GetMusicDirectoryJsonResponse {
    GetMusicDirectoryBody {
        directory: get_music_directory(&database.pool, req.user_id, req.params.id).await?,
    }
    .into()
}
