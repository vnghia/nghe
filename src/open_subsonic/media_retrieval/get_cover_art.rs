use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel::{
    ExpressionMethods, OptionalExtension, PgSortExpressionMethods, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::add_common_validate;
use nghe_types::id::MediaType;
use uuid::Uuid;

use crate::config::ArtConfig;
use crate::models::*;
use crate::open_subsonic::permission::with_permission;
use crate::open_subsonic::StreamResponse;
use crate::utils::fs::LocalPath;
use crate::{Database, DatabasePool, OSError, ServerError};

add_common_validate!(GetCoverArtParams);

pub async fn get_song_cover_art(
    pool: &DatabasePool,
    user_id: Uuid,
    cover_art_id: Uuid,
    song_art_dir: impl AsRef<LocalPath>,
) -> Result<StreamResponse> {
    let song_cover_art = songs::table
        .inner_join(cover_arts::table)
        .filter(with_permission(user_id))
        .filter(cover_arts::id.eq(cover_art_id))
        .select(cover_arts::CoverArt::as_select())
        .get_result(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song cover art".into()))?;
    StreamResponse::try_from_path(
        song_cover_art.to_path(song_art_dir),
        None,
        song_cover_art.file_size as u64,
        false,
    )
    .await
}

pub async fn get_album_cover_art(
    pool: &DatabasePool,
    user_id: Uuid,
    album_id: Uuid,
    song_art_dir: impl AsRef<LocalPath>,
) -> Result<StreamResponse> {
    let album_cover_art = songs::table
        .inner_join(cover_arts::table)
        .filter(with_permission(user_id))
        .filter(songs::album_id.eq(album_id))
        .order((
            // First disc, first track, latest song and lastly smallest cover art id if any
            // difference.
            songs::disc_number.asc().nulls_last(),
            songs::track_number.asc().nulls_last(),
            songs::year.desc().nulls_last(),
            cover_arts::file_size.asc().nulls_last(),
            // Add this to ensure that album cover art will be deterministic.
            cover_arts::file_hash.asc().nulls_last(),
        ))
        .select(cover_arts::CoverArt::as_select())
        .first(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Album cover art".into()))?;
    StreamResponse::try_from_path(
        album_cover_art.to_path(song_art_dir),
        None,
        album_cover_art.file_size as u64,
        false,
    )
    .await
}

pub async fn get_artist_cover_art(
    pool: &DatabasePool,
    cover_art_id: Uuid,
    artist_art_dir: impl AsRef<LocalPath>,
) -> Result<StreamResponse> {
    let artist_cover_art = artists::table
        .inner_join(cover_arts::table)
        .filter(cover_arts::id.eq(cover_art_id))
        .select(cover_arts::CoverArt::as_select())
        .get_result(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Artist cover art".into()))?;
    StreamResponse::try_from_path(
        artist_cover_art.to_path(artist_art_dir),
        None,
        artist_cover_art.file_size as u64,
        false,
    )
    .await
}

pub async fn get_cover_art_handler(
    State(database): State<Database>,
    Extension(art_config): Extension<ArtConfig>,
    req: GetCoverArtRequest,
) -> Result<StreamResponse, ServerError> {
    match req.params.id.t {
        Some(MediaType::Song) if let Some(song_art_dir) = art_config.song_dir => {
            get_song_cover_art(&database.pool, req.user_id, req.params.id.id, &song_art_dir).await
        }
        Some(MediaType::Album) if let Some(song_art_dir) = art_config.song_dir => {
            get_album_cover_art(&database.pool, req.user_id, req.params.id.id, &song_art_dir).await
        }
        Some(MediaType::Aritst) if let Some(artist_art_dir) = art_config.artist_dir => {
            get_artist_cover_art(&database.pool, req.params.id.id, &artist_art_dir).await
        }
        _ => Err(anyhow::anyhow!(OSError::NotFound("Cover art".into()))),
    }
    .map_err(ServerError)
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use fake::{Fake, Faker};

    use super::*;
    use crate::open_subsonic::scan::test::upsert_album;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::http::to_bytes;
    use crate::utils::test::picture::fake;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_song_cover_art() {
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        let picture = fake(true);
        infra
            .add_songs(0, vec![SongTag { picture: picture.clone(), ..Faker.fake() }])
            .await
            .scan(.., None)
            .await;

        let download_bytes = to_bytes(
            get_song_cover_art(
                infra.pool(),
                infra.user_id(0),
                infra.song_cover_art_ids(..).await[0],
                infra.fs.art_config.song_dir.as_ref().unwrap(),
            )
            .await
            .unwrap()
            .into_response(),
        )
        .await
        .to_vec();
        let cover_art_bytes = picture.as_ref().unwrap().data();
        assert_eq!(download_bytes, cover_art_bytes);
    }

    #[tokio::test]
    async fn test_get_album_cover_art() {
        let album_name = "album";
        let picture = fake(true);
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        album: album_name.into(),
                        picture: picture.clone(),
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let download_bytes = to_bytes(
            get_album_cover_art(
                infra.pool(),
                infra.user_id(0),
                album_id,
                infra.fs.art_config.song_dir.as_ref().unwrap(),
            )
            .await
            .unwrap()
            .into_response(),
        )
        .await
        .to_vec();
        let cover_art_bytes = picture.as_ref().unwrap().data();
        assert_eq!(download_bytes, cover_art_bytes);
    }

    #[tokio::test]
    async fn test_get_album_cover_art_no_picture() {
        let album_name = "album";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag { album: album_name.into(), picture: None, ..Faker.fake() })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        assert!(
            get_album_cover_art(
                infra.pool(),
                infra.user_id(0),
                album_id,
                infra.fs.art_config.song_dir.as_ref().unwrap()
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_get_album_cover_art_deny() {
        let album_name = "album";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        infra
            .add_n_song(0, n_song)
            .await
            .add_songs(
                1,
                (0..n_song)
                    .map(|_| SongTag {
                        album: album_name.into(),
                        picture: fake(true),
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;
        infra.remove_permissions(.., 1..).await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        assert!(
            get_album_cover_art(
                infra.pool(),
                infra.user_id(0),
                album_id,
                infra.fs.art_config.song_dir.as_ref().unwrap()
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_get_album_cover_art_partial() {
        let album_name = "album";
        let picture1 = fake(true);
        let picture2 = fake(true);
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        album: album_name.into(),
                        picture: picture1.clone(),
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .add_songs(
                1,
                (0..n_song)
                    .map(|_| SongTag {
                        album: album_name.into(),
                        picture: picture2.clone(),
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;
        infra.remove_permissions(.., 1..).await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let download_bytes = to_bytes(
            get_album_cover_art(
                infra.pool(),
                infra.user_id(0),
                album_id,
                infra.fs.art_config.song_dir.as_ref().unwrap(),
            )
            .await
            .unwrap()
            .into_response(),
        )
        .await
        .to_vec();
        let cover_art_bytes = picture1.as_ref().unwrap().data();
        assert_eq!(download_bytes, cover_art_bytes);
    }
}
