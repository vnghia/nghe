use anyhow::Result;
use axum::extract::State;
use diesel::dsl::sql;
use diesel::{sql_types, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::with_permission;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(GetArtistParams);
add_axum_response!(GetArtistBody);

async fn get_artist_and_album_ids(
    pool: &DatabasePool,
    user_id: Uuid,
    artist_id: Uuid,
) -> Result<(ArtistAlbumCountId3Db, Vec<Uuid>)> {
    if let Some(res) = get_album_artist_id3_db()
        .filter(with_permission(user_id))
        .filter(artists::id.eq(artist_id))
        .select((
            ArtistAlbumCountId3Db::as_select(),
            sql::<sql_types::Array<sql_types::Uuid>>(
                "array_agg(distinct(songs.album_id)) album_ids",
            ),
        ))
        .first::<(ArtistAlbumCountId3Db, Vec<Uuid>)>(&mut pool.get().await?)
        .await
        .optional()?
    {
        Ok(res)
    } else {
        Ok((
            get_no_album_artist_id3_db()
                .filter(with_permission(user_id))
                .filter(artists::id.eq(artist_id))
                .first(&mut pool.get().await?)
                .await
                .optional()?
                .ok_or_else(|| OSError::NotFound("Artist".into()))?
                .into(),
            vec![],
        ))
    }
}

async fn get_basic_albums(
    pool: &DatabasePool,
    user_id: Uuid,
    album_ids: &[Uuid],
) -> Result<Vec<BasicAlbumId3Db>> {
    get_basic_album_id3_db()
        .filter(with_permission(user_id))
        .filter(albums::id.eq_any(album_ids))
        .get_results::<BasicAlbumId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_artist(
    pool: &DatabasePool,
    user_id: Uuid,
    artist_id: Uuid,
) -> Result<ArtistId3WithAlbums> {
    let (artist, album_ids) = get_artist_and_album_ids(pool, user_id, artist_id).await?;
    let basic_albums = get_basic_albums(pool, user_id, &album_ids).await?;

    Ok(ArtistId3WithAlbums {
        artist: artist.into(),
        albums: basic_albums.into_iter().map(BasicAlbumId3Db::into).collect(),
    })
}

pub async fn get_artist_handler(
    State(database): State<Database>,
    req: GetArtistRequest,
) -> GetArtistJsonResponse {
    Ok(axum::Json(
        GetArtistBody { artist: get_artist(&database.pool, req.user_id, req.params.id).await? }
            .into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::Rng;

    use super::*;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_artist_own_albums() {
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
            .await
            .scan(.., None)
            .await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let (artist, album_ids) =
            get_artist_and_album_ids(infra.pool(), infra.user_id(0), artist_id).await.unwrap();
        let album_count = artist.album_count;
        let album_ids = album_ids.into_iter().sorted().collect_vec();
        assert_eq!(album_count as usize, album_ids.len());
        assert_eq!(album_ids, infra.album_ids(&infra.album_no_ids(..)).await);
    }

    #[tokio::test]
    async fn test_get_artist_albums_compilation() {
        let artist_name = "artist";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        artists: vec![artist_name.into()],
                        compilation: true,
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let (artist, album_ids) =
            get_artist_and_album_ids(infra.pool(), infra.user_id(0), artist_id).await.unwrap();
        let album_count = artist.album_count;
        let album_ids = album_ids.into_iter().sorted().collect_vec();
        assert_eq!(album_count as usize, album_ids.len());
        assert_eq!(album_ids, infra.album_ids(&infra.album_no_ids(..)).await);
    }

    #[tokio::test]
    async fn test_get_artist_albums_no_compilation() {
        let artist_name = "artist";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        artists: vec![artist_name.into()],
                        album_artists: vec![Faker.fake::<String>().into()],
                        compilation: false,
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let (artist, album_ids) =
            get_artist_and_album_ids(infra.pool(), infra.user_id(0), artist_id).await.unwrap();
        let album_count = artist.album_count;
        assert_eq!(album_count, 0);
        assert!(album_ids.is_empty());
    }

    #[tokio::test]
    async fn test_get_artist_distinct_albums_compilation() {
        let artist_name = "artist";
        let album_names = ["album1", "album2"];
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|i| SongTag {
                        artists: vec![artist_name.into()],
                        album: if i < 5 { album_names[0].into() } else { album_names[1].into() },
                        compilation: true,
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let (artist, album_ids) =
            get_artist_and_album_ids(infra.pool(), infra.user_id(0), artist_id).await.unwrap();
        let album_count = artist.album_count;
        let album_ids = album_ids.into_iter().sorted().collect_vec();
        assert_eq!(album_count as usize, album_ids.len());
        assert_eq!(album_ids.len(), album_names.len());
        assert_eq!(album_ids, infra.album_ids(&infra.album_no_ids(..)).await);
    }

    #[tokio::test]
    async fn test_get_artist_partial_music_folders_compilation() {
        let artist_name = "artist";
        let n_folder = 2_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        for i in 0..n_folder {
            infra
                .add_songs(
                    i,
                    (0..n_song)
                        .map(|_| SongTag {
                            artists: vec![artist_name.into()],
                            compilation: true,
                            ..Faker.fake()
                        })
                        .collect(),
                )
                .await;
        }
        infra.scan(.., None).await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        let music_folder_idx = rand::thread_rng().gen_range(0..infra.music_folders.len());
        infra.remove_permission(None, None).await.add_permission(None, music_folder_idx).await;

        let (artist, album_ids) =
            get_artist_and_album_ids(infra.pool(), infra.user_id(0), artist_id).await.unwrap();
        let album_count = artist.album_count;
        let album_ids = album_ids.into_iter().sorted().collect_vec();
        assert_eq!(album_count as usize, album_ids.len());
        assert_eq!(
            album_ids,
            infra.album_ids(&infra.album_no_ids(music_folder_idx..=music_folder_idx)).await
        );
    }

    #[tokio::test]
    async fn test_get_artist_partial_music_folders_no_compilation() {
        let artist_name = "artist";
        let n_folder = 2_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        artists: vec![artist_name.into()],
                        album_artists: vec![Faker.fake::<String>().into()],
                        compilation: false,
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .add_songs(
                1,
                (0..n_song)
                    .map(|_| SongTag { album_artists: vec![artist_name.into()], ..Faker.fake() })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;
        infra.remove_permission(None, None).await.add_permissions(.., ..1).await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);

        let (artist, album_ids) =
            get_artist_and_album_ids(infra.pool(), infra.user_id(0), artist_id).await.unwrap();
        let album_count = artist.album_count;
        assert_eq!(album_count, 0);
        assert!(album_ids.is_empty());
    }

    #[tokio::test]
    async fn test_get_artist_albums_deny_music_folders() {
        // The artist exists but does not reside in the allowed music folders.
        let artist_name = "artist";
        let n_folder = 2_usize;
        let n_scan_folder = 1_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        infra
            .add_n_song(0, n_song)
            .await
            .add_songs(
                1,
                (0..n_song)
                    .map(|_| SongTag { artists: vec![artist_name.into()], ..Faker.fake() })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;
        infra.remove_permission(None, None).await.add_permissions(.., ..n_scan_folder).await;

        let artist_id =
            upsert_artists(infra.pool(), &[], &[artist_name.into()]).await.unwrap().remove(0);
        assert!(matches!(
            get_artist_and_album_ids(infra.pool(), infra.user_id(0), artist_id)
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_get_basic_albums() {
        let album_names = ["album1", "album2"];
        let n_folder = 3_usize;
        let n_song = 10_usize;
        let n_diff = 3_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag { album: album_names[0].into(), ..Faker.fake() })
                    .collect(),
            )
            .await
            .add_songs(
                1,
                (0..n_song + n_diff)
                    .map(|_| SongTag { album: album_names[1].into(), ..Faker.fake() })
                    .collect(),
            )
            .await
            .add_songs(
                2,
                (0..n_song - n_diff)
                    .map(|_| SongTag { album: album_names[1].into(), ..Faker.fake() })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;
        infra.remove_permission(None, None).await.add_permissions(.., ..2).await;

        let basic_albums = get_basic_albums(
            infra.pool(),
            infra.user_id(0),
            &infra.album_ids(&infra.album_no_ids(..)).await,
        )
        .await
        .unwrap()
        .into_iter()
        .sorted_by(|a, b| a.no_id.name.cmp(&b.no_id.name))
        .collect_vec();

        assert_eq!(basic_albums[0].song_count as usize, n_song);
        assert_eq!(basic_albums[1].song_count as usize, n_song + n_diff);
    }
}
