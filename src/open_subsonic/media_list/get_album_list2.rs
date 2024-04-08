use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, PgSortExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{
    add_common_convert, add_common_validate, add_count_offset, add_permission_filter,
    wrap_subsonic_response,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::query::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::common::sql;
use crate::open_subsonic::permission::check_permission;
use crate::{Database, DatabasePool, OSError};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GetAlbumListType {
    Random,
    Newest,
    Recent,
    ByYear,
    ByGenre,
    AlphabeticalByName,
}

#[add_common_convert]
#[derive(Debug)]
pub struct GetAlbumList2Params {
    #[serde(rename = "type")]
    list_type: GetAlbumListType,
    #[serde(rename = "size")]
    count: Option<i64>,
    offset: Option<i64>,
    #[serde(rename = "musicFolderId")]
    music_folder_ids: Option<Vec<Uuid>>,
    // By Year
    from_year: Option<i16>,
    to_year: Option<i16>,
    // By Genre
    genre: Option<String>,
}
add_common_validate!(GetAlbumList2Params);

#[derive(Debug, Serialize)]
struct AlbumList2 {
    album: Vec<AlbumId3>,
}

#[wrap_subsonic_response]
struct GetAlbumList2Body {
    album_list2: AlbumList2,
}

pub async fn get_album_list2(
    pool: &DatabasePool,
    user_id: Uuid,
    GetAlbumList2Params {
      list_type, count, offset, music_folder_ids, from_year, to_year, genre
    }: GetAlbumList2Params,
) -> Result<Vec<AlbumId3>> {
    let albums = match list_type {
        GetAlbumListType::Random =>
        {
            #[add_permission_filter]
            #[add_count_offset]
            get_album_id3_db()
                .order(sql::random())
                .get_results::<AlbumId3Db>(&mut pool.get().await?)
                .await?
        }
        GetAlbumListType::Newest =>
        {
            #[add_permission_filter]
            #[add_count_offset]
            get_album_id3_db()
                .order(albums::year.desc().nulls_last())
                .get_results::<AlbumId3Db>(&mut pool.get().await?)
                .await?
        }
        GetAlbumListType::Recent =>
        {
            #[add_permission_filter]
            #[add_count_offset]
            get_album_id3_db()
                .order(albums::created_at.desc())
                .get_results::<AlbumId3Db>(&mut pool.get().await?)
                .await?
        }
        GetAlbumListType::ByYear => {
            let from_year = from_year.ok_or_else(|| {
                OSError::InvalidParameter("from year is missing when list by year".into())
            })?;
            let to_year = to_year.ok_or_else(|| {
                OSError::InvalidParameter("to year is missing when list by year".into())
            })?;
            if from_year < to_year {
                #[add_permission_filter]
                #[add_count_offset]
                get_album_id3_db()
                    .filter(albums::year.is_not_null())
                    .filter(albums::year.ge(from_year))
                    .filter(albums::year.le(to_year))
                    .order(albums::year.asc())
                    .get_results::<AlbumId3Db>(&mut pool.get().await?)
                    .await?
            } else {
                #[add_permission_filter]
                #[add_count_offset]
                get_album_id3_db()
                    .filter(albums::year.is_not_null())
                    .filter(albums::year.le(from_year))
                    .filter(albums::year.ge(to_year))
                    .order(albums::year.desc())
                    .get_results::<AlbumId3Db>(&mut pool.get().await?)
                    .await?
            }
        }
        GetAlbumListType::ByGenre => {
            let genre = genre.ok_or_else(|| {
                OSError::InvalidParameter("genre is missing when list by genre".into())
            })?;
            #[add_permission_filter]
            #[add_count_offset]
            get_album_id3_db()
                .filter(genres::value.eq(&genre))
                .order(albums::name.asc())
                .get_results::<AlbumId3Db>(&mut pool.get().await?)
                .await?
        }
        GetAlbumListType::AlphabeticalByName =>
        {
            #[add_permission_filter]
            #[add_count_offset]
            get_album_id3_db()
                .order(albums::name.asc())
                .get_results::<AlbumId3Db>(&mut pool.get().await?)
                .await?
        }
    };

    stream::iter(albums).then(|v| async move { v.into_res(pool).await }).try_collect().await
}

pub async fn get_album_list2_handler(
    State(database): State<Database>,
    req: GetAlbumList2Request,
) -> GetAlbumList2JsonResponse {
    check_permission(&database.pool, req.user_id, &req.params.music_folder_ids).await?;

    GetAlbumList2Body {
        album_list2: AlbumList2 {
            album: get_album_list2(&database.pool, req.user_id, req.params).await?,
        },
    }
    .into()
}
