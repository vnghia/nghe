use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use nghe_proc_macros::{add_axum_response, add_common_validate, add_permission_filter};
use uuid::Uuid;

use crate::config::ArtistIndexConfig;
use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::check_permission;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(GetArtistsParams);
add_axum_response!(GetArtistsBody);

async fn get_indexed_artists(
    pool: &DatabasePool,
    user_id: Uuid,
    music_folder_ids: &Option<Vec<Uuid>>,
) -> Result<Vec<(String, ArtistId3Db)>> {
    #[add_permission_filter]
    get_basic_artist_id3_db()
        .select((artists::index, ArtistId3Db::as_select()))
        .get_results::<(String, ArtistId3Db)>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_artists(
    pool: &DatabasePool,
    user_id: Uuid,
    music_folder_ids: &Option<Vec<Uuid>>,
) -> Result<Indexes> {
    check_permission(pool, user_id, music_folder_ids).await?;

    let ignored_articles = configs::table
        .select(configs::text)
        .filter(configs::key.eq(ArtistIndexConfig::IGNORED_ARTICLES_CONFIG_KEY))
        .first::<Option<String>>(&mut pool.get().await?)
        .await?
        .ok_or_else(|| OSError::NotFound("Ignored articles".into()))?;

    let index = get_indexed_artists(pool, user_id, music_folder_ids)
        .await?
        .into_iter()
        .into_group_map()
        .into_iter()
        .map(|(k, v)| Index { name: k, artists: v.into_iter().map(ArtistId3Db::into).collect() })
        .collect_vec();

    Ok(Indexes { ignored_articles, index })
}

pub async fn get_artists_handler(
    State(database): State<Database>,
    req: GetArtistsRequest,
) -> GetArtistsJsonResponse {
    Ok(axum::Json(
        GetArtistsBody {
            artists: get_artists(&database.pool, req.user_id, &req.params.music_folder_ids).await?,
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_indexed_artists() {
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, n_song).scan(.., None).await;

        let artist_ids = get_indexed_artists(infra.pool(), infra.user_id(0), &None)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, artist)| artist.basic.id)
            .sorted()
            .collect_vec();
        assert_eq!(artist_ids, infra.artist_ids(&infra.artist_no_ids(..)).await);
    }

    #[tokio::test]
    async fn test_get_indexed_song_artists() {
        let artist_name = "artist";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag { artists: vec![artist_name.into()], ..Faker.fake() })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let artist_ids = get_indexed_artists(infra.pool(), infra.user_id(0), &None)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, artist)| artist.basic.id)
            .sorted()
            .collect_vec();
        assert!(artist_ids.contains(&artist_id));
    }

    #[tokio::test]
    async fn test_get_indexed_album_artists() {
        let artist_name = "artist";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag { album_artists: vec![artist_name.into()], ..Faker.fake() })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let artist_ids = get_indexed_artists(infra.pool(), infra.user_id(0), &None)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, artist)| artist.basic.id)
            .sorted()
            .collect_vec();
        assert!(artist_ids.contains(&artist_id));
    }

    #[tokio::test]
    async fn test_get_indexed_artists_multiple_music_folders() {
        let artist_name = "artist";
        let n_folder = 5_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        (0..n_folder).for_each(|i| {
            infra.add_songs(
                i,
                (0..n_song)
                    .map(|_| SongTag { artists: vec![artist_name.into()], ..Faker.fake() })
                    .collect(),
            );
        });
        infra.scan(.., None).await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let artist_ids = get_indexed_artists(infra.pool(), infra.user_id(0), &None)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, artist)| artist.basic.id)
            .sorted()
            .collect_vec();
        assert!(artist_ids.contains(&artist_id));
    }

    #[tokio::test]
    async fn test_get_indexed_artists_deny_music_folders() {
        let artist_name = "artist";
        let n_folder = 5_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        (0..n_folder).for_each(|i| {
            infra.add_songs(
                i,
                (0..n_song)
                    .map(|_| SongTag {
                        artists: if i >= 2 {
                            vec![artist_name.into()]
                        } else {
                            artists::ArtistNoId::fake_vec(1..2)
                        },
                        ..Faker.fake()
                    })
                    .collect(),
            );
        });
        infra.scan(.., None).await;
        infra.remove_permission(None, None).await.add_permissions(.., 0..2).await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let artist_ids = get_indexed_artists(infra.pool(), infra.user_id(0), &None)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, artist)| artist.basic.id)
            .sorted()
            .collect_vec();
        assert!(!artist_ids.contains(&artist_id));
    }

    #[tokio::test]
    async fn test_get_artists() {
        let n_folder = 5_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        (0..n_folder).for_each(|i| {
            infra.add_n_song(i, n_song);
        });
        infra.scan(.., None).await;
        infra.remove_permission(None, None).await.add_permissions(.., 0..2).await;

        assert!(get_artists(infra.pool(), infra.user_id(0), &None).await.is_ok());
        assert!(matches!(
            get_artists(infra.pool(), infra.user_id(0), &Some(infra.music_folder_ids(..)))
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::Forbidden(_)
        ));
    }
}
