use anyhow::Result;
use axum::extract::State;
use diesel::dsl::{max, sum};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{
    add_axum_response, add_common_validate, add_count_offset, add_permission_filter,
};
use nghe_types::id3::*;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::sql;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::check_permission;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(GetAlbumList2Params);
add_axum_response!(GetAlbumList2Body);

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
                .order(albums::created_at.desc())
                .get_results::<AlbumId3Db>(&mut pool.get().await?)
                .await?
        }
        GetAlbumListType::Frequent =>
        {
            #[add_permission_filter]
            #[add_count_offset]
            get_album_id3_db()
                .inner_join(playbacks::table)
                .filter(playbacks::user_id.eq(user_id))
                .order(sum(playbacks::count).desc())
                .get_results::<AlbumId3Db>(&mut pool.get().await?)
                .await?
        }
        GetAlbumListType::Recent =>
        {
            #[add_permission_filter]
            #[add_count_offset]
            get_album_id3_db()
                .inner_join(playbacks::table)
                .filter(playbacks::user_id.eq(user_id))
                .order(max(playbacks::updated_at).desc())
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

    stream::iter(albums).then(|v| async move { v.into(pool).await }).try_collect().await
}

pub async fn get_album_list2_handler(
    State(database): State<Database>,
    req: GetAlbumList2Request,
) -> GetAlbumList2JsonResponse {
    check_permission(&database.pool, req.user_id, &req.params.music_folder_ids).await?;

    Ok(axum::Json(
        GetAlbumList2Body {
            album_list2: AlbumList2 {
                album: get_album_list2(&database.pool, req.user_id, req.params).await?,
            },
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_album() {
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, 20).scan(.., None).await;
        for list_type in GetAlbumListType::iter() {
            get_album_list2(
                infra.pool(),
                infra.user_id(0),
                GetAlbumList2Params {
                    list_type,
                    count: None,
                    offset: None,
                    music_folder_ids: None,
                    from_year: Some(1000),
                    to_year: Some(2000),
                    genre: Some("genre".into()),
                },
            )
            .await
            .unwrap();
        }
    }
}
