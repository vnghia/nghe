use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::lists::get_starred2::Starred2;
pub use nghe_api::lists::get_starred2::{Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{id3, star_albums, star_artists, star_songs};

// TODO: Rethink unchecked while querying starred. We could filter before join but it requires a
// `user_id`. Or maybe `with_user_id_unchecked`.
#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    #[check_music_folder]
    {
        let artist = id3::artist::query::with_user_id(user_id)
            .filter(star_artists::user_id.eq(user_id))
            .get_results(&mut database.get().await?)
            .await?;
        let album = id3::album::short::query::with_user_id(user_id)
            .filter(star_albums::user_id.eq(user_id))
            .get_results(&mut database.get().await?)
            .await?;
        let song = id3::song::short::query::with_user_id(user_id)
            .filter(star_songs::user_id.eq(user_id))
            .get_results(&mut database.get().await?)
            .await?;

        Ok(Response {
            starred2: Starred2 {
                artist: artist.into_iter().map(id3::artist::Artist::try_into).try_collect()?,
                album: album.into_iter().map(id3::album::short::Short::try_into).try_collect()?,
                song: song.into_iter().map(id3::song::short::Short::try_into).try_collect()?,
            },
        })
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::route::media_annotation::star;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_star_artist(
        #[future(awt)]
        #[with(2, 1)]
        mock: Mock,
    ) {
        let mut music_folder = mock.music_folder(0).await;
        let user_id_star = mock.user_id(0).await;
        let user_id = mock.user_id(1).await;

        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;
        music_folder.add_audio_artist([artist.clone()], [], false, 1).await;

        star::handler(mock.database(), user_id_star, star::Request {
            artist_ids: Some(vec![artist_id]),
            ..Default::default()
        })
        .await
        .unwrap();

        {
            let starred =
                handler(mock.database(), user_id_star, Request::default()).await.unwrap().starred2;
            assert!(starred.song.is_empty());
            assert!(starred.album.is_empty());
            assert_eq!(
                starred.artist.into_iter().map(|artist| artist.required.id).collect::<Vec<_>>(),
                vec![artist_id]
            );
        }

        {
            let starred =
                handler(mock.database(), user_id, Request::default()).await.unwrap().starred2;
            assert!(starred.song.is_empty());
            assert!(starred.album.is_empty());
            assert!(starred.artist.is_empty());
        }
    }
}
