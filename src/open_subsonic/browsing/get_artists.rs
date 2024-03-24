use anyhow::Result;
use axum::extract::State;
use diesel::dsl::count_distinct;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use crate::config::ArtistIndexConfig;
use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::common::music_folder::check_user_music_folder_ids;
use crate::{Database, DatabasePool, OSError};

#[add_validate]
#[derive(Debug)]
pub struct GetArtistsParams {
    #[serde(rename = "musicFolderId")]
    music_folder_ids: Option<Vec<Uuid>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub name: String,
    #[serde(rename = "artist")]
    pub artists: Vec<ArtistId3>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Indexes {
    pub ignored_articles: String,
    pub index: Vec<Index>,
}

#[wrap_subsonic_response]
pub struct GetArtistsBody {
    pub artists: Indexes,
}

async fn get_indexed_artists(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
) -> Result<Vec<(String, BasicArtistId3Db)>> {
    artists::table
        .left_join(songs_album_artists::table)
        .left_join(songs_artists::table)
        .inner_join(songs::table.on(
            songs::id.eq(songs_album_artists::song_id).or(songs::id.eq(songs_artists::song_id)),
        ))
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .group_by(artists::id)
        .having(count_distinct(songs::album_id).gt(0))
        .select((artists::index, BasicArtistId3Db::as_select()))
        .get_results::<(String, BasicArtistId3Db)>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_artists(
    pool: &DatabasePool,
    user_id: Uuid,
    music_folder_ids: Option<Vec<Uuid>>,
) -> Result<Indexes> {
    let music_folder_ids =
        check_user_music_folder_ids(pool, &user_id, music_folder_ids.map(|v| v.into())).await?;

    let ignored_articles = configs::table
        .select(configs::text)
        .filter(configs::key.eq(ArtistIndexConfig::IGNORED_ARTICLES_CONFIG_KEY))
        .first::<Option<String>>(&mut pool.get().await?)
        .await?
        .ok_or_else(|| OSError::NotFound("Ignored articles".into()))?;

    let index = get_indexed_artists(pool, &music_folder_ids)
        .await?
        .into_iter()
        .into_group_map()
        .into_iter()
        .map(|(k, v)| Index { name: k, artists: v.into_iter().map(|v| v.into_res()).collect() })
        .collect_vec();

    Ok(Indexes { ignored_articles, index })
}

pub async fn get_artists_handler(
    State(database): State<Database>,
    req: GetArtistsRequest,
) -> GetArtistsJsonResponse {
    GetArtistsBody {
        artists: get_artists(&database.pool, req.user_id, req.params.music_folder_ids).await?,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::media::song_paths_to_artist_ids;
    use crate::utils::test::setup::TestInfra;

    #[tokio::test]
    async fn test_get_artists() {
        let n_song = 10_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(&[n_song], None).await;
        let music_folder_ids = test_infra.music_folder_ids(..);

        let artist_ids = get_indexed_artists(test_infra.pool(), &music_folder_ids)
            .await
            .unwrap()
            .into_iter()
            .map(|(_, artist)| artist.id)
            .sorted()
            .collect_vec();
        let artist_fs_ids = song_paths_to_artist_ids(test_infra.pool(), &song_fs_infos).await;

        assert_eq!(artist_ids, artist_fs_ids);
    }

    #[tokio::test]
    async fn test_get_song_artists() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (test_infra, _) = TestInfra::setup_songs(
            &[n_song],
            (0..n_song)
                .map(|_| SongTag { artists: vec![artist_name.to_owned()], ..Faker.fake() })
                .collect_vec(),
        )
        .await;
        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);

        let artist_ids =
            get_indexed_artists(test_infra.pool(), &test_infra.music_folder_ids(0..=0))
                .await
                .unwrap()
                .into_iter()
                .map(|(_, artist)| artist.id)
                .sorted()
                .collect_vec();

        assert!(artist_ids.contains(&artist_id));
    }

    #[tokio::test]
    async fn test_get_album_artists() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (test_infra, _) = TestInfra::setup_songs(
            &[n_song],
            (0..n_song)
                .map(|_| SongTag { album_artists: vec![artist_name.to_owned()], ..Faker.fake() })
                .collect_vec(),
        )
        .await;
        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);

        let artist_ids =
            get_indexed_artists(test_infra.pool(), &test_infra.music_folder_ids(0..=0))
                .await
                .unwrap()
                .into_iter()
                .map(|(_, artist)| artist.id)
                .sorted()
                .collect_vec();

        assert!(artist_ids.contains(&artist_id));
    }

    #[tokio::test]
    async fn test_get_artists_multiple_music_folders() {
        let artist_name = "artist";
        let n_folder = 5_usize;
        let n_song = 10_usize;

        let (test_infra, _) = TestInfra::setup_songs(
            &vec![n_song; n_folder],
            (0..n_folder * n_song)
                .map(|_| SongTag { artists: vec![artist_name.to_owned()], ..Faker.fake() })
                .collect_vec(),
        )
        .await;
        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);

        let artist_ids =
            get_indexed_artists(test_infra.pool(), &test_infra.music_folder_ids(0..=0))
                .await
                .unwrap()
                .into_iter()
                .map(|(_, artist)| artist.id)
                .sorted()
                .collect_vec();

        assert!(artist_ids.contains(&artist_id));
    }

    #[tokio::test]
    async fn test_get_artists_deny_music_folders() {
        let artist_name = "artist";
        let n_folder = 5_usize;
        let n_song = 10_usize;

        let (test_infra, _) = TestInfra::setup_songs(
            &vec![n_song; n_folder],
            (0..n_folder * n_song)
                .map(|i| SongTag {
                    artists: if i >= 2 * n_song {
                        vec![artist_name.to_owned()]
                    } else {
                        fake::vec![String; 1..2]
                    },
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;
        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);

        let artist_ids = get_indexed_artists(test_infra.pool(), &test_infra.music_folder_ids(0..2))
            .await
            .unwrap()
            .into_iter()
            .map(|(_, artist)| artist.id)
            .sorted()
            .collect_vec();

        assert!(!artist_ids.contains(&artist_id));
    }
}
