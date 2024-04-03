use std::path::Path;

use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::add_validate;
use uuid::Uuid;

use super::super::common::id::{MediaType, MediaTypedId};
use crate::config::ArtConfig;
use crate::models::*;
use crate::open_subsonic::permission::with_permission;
use crate::open_subsonic::StreamResponse;
use crate::{Database, DatabasePool, OSError, ServerError};

#[add_validate]
#[derive(Debug)]
pub struct GetCoverArtParams {
    id: MediaTypedId,
}

pub async fn get_song_cover_art<P: AsRef<Path>>(
    pool: &DatabasePool,
    user_id: Uuid,
    cover_art_id: Uuid,
    song_art_dir: P,
) -> Result<StreamResponse> {
    let song_cover_art = songs::table
        .inner_join(song_cover_arts::table)
        .filter(with_permission(user_id))
        .filter(song_cover_arts::id.eq(cover_art_id))
        .select(song_cover_arts::SongCoverArt::as_select())
        .get_result(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song cover art".into()))?;
    StreamResponse::try_from_path(song_cover_art.to_path(song_art_dir)).await
}

pub async fn get_cover_art_handler(
    State(database): State<Database>,
    Extension(art_config): Extension<ArtConfig>,
    req: GetCoverArtRequest,
) -> Result<StreamResponse, ServerError> {
    match req.params.id.t {
        Some(MediaType::Song) if let Some(song_path) = art_config.song_path => {
            get_song_cover_art(&database.pool, req.user_id, req.params.id.id, &song_path)
                .await
                .map_err(ServerError)
        }
        _ => Err(anyhow::anyhow!(OSError::NotFound("Cover art".into())).into()),
    }
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use fake::{Fake, Faker};

    use super::*;
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
            .scan(.., None)
            .await;

        let download_bytes = to_bytes(
            get_song_cover_art(
                infra.pool(),
                infra.user_id(0),
                infra.song_cover_art_ids(..).await[0],
                infra.fs.art_config.song_path.as_ref().unwrap(),
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
}
