use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::id3::*;
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetPlaylistsParams);
add_axum_response!(GetPlaylistsBody);

pub async fn get_playlists(pool: &DatabasePool, user_id: Uuid) -> Result<Vec<PlaylistId3Db>> {
    get_playlist_id3_db()
        .filter(playlists_users::user_id.eq(user_id))
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_playlists_handler(
    State(database): State<Database>,
    req: GetPlaylistsRequest,
) -> GetPlaylistsJsonResponse {
    Ok(axum::Json(
        GetPlaylistsBody {
            playlists: GetPlaylists {
                playlist: get_playlists(&database.pool, req.user_id)
                    .await?
                    .into_iter()
                    .map(PlaylistId3Db::into)
                    .collect(),
            },
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use nghe_types::playlists::create_playlist::CreatePlaylistParams;
    use rand::prelude::SliceRandom;

    use super::super::create_playlist::create_playlist;
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_empty_playlists() {
        let infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        let playlists = get_playlists(infra.pool(), infra.user_id(0)).await.unwrap();
        assert!(playlists.is_empty());
    }

    #[tokio::test]
    async fn test_get_playlists() {
        let mut infra =
            Infra::new().await.add_user(None).await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, 10).scan(.., None).await;
        let song_fs_ids = infra.song_ids(..).await;

        for user_id in infra.user_ids(..) {
            let n_playlist = (1..3).fake();
            let mut db_playlists = vec![];
            for _ in 0..n_playlist {
                db_playlists.push(
                    create_playlist(
                        infra.pool(),
                        user_id,
                        &CreatePlaylistParams {
                            name: Some(Faker.fake()),
                            playlist_id: None,
                            song_ids: song_fs_ids
                                .choose_multiple(&mut rand::thread_rng(), (2..4).fake())
                                .copied()
                                .collect(),
                        },
                    )
                    .await
                    .unwrap()
                    .playlist,
                );
            }
            let playlists = get_playlists(infra.pool(), user_id).await.unwrap();
            assert_eq!(
                playlists.into_iter().sorted_by_key(|p| p.basic.id).collect_vec(),
                db_playlists.into_iter().sorted_by_key(|p| p.basic.id).collect_vec()
            );
        }
    }
}
