use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::lists::get_starred2::Starred2;
pub use nghe_api::lists::get_starred2::{Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{id3, star_albums, star_artists, star_songs};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    #[check_music_folder]
    {
        let artist = id3::artist::query::with_user_id(user_id)
            .filter(star_artists::user_id.is_not_null())
            .get_results(&mut database.get().await?)
            .await?;
        let album = id3::album::short::query::with_user_id(user_id)
            .filter(star_albums::user_id.is_not_null())
            .get_results(&mut database.get().await?)
            .await?;
        let song = id3::song::short::query::with_user_id(user_id)
            .filter(star_songs::user_id.is_not_null())
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
    use crate::route::browsing::{get_album, get_artist, get_artists};
    use crate::route::media_annotation::star;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_star_artist(
        #[future(awt)]
        #[with(2, 1)]
        mock: Mock,
    ) {
        let database = mock.database();
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
                handler(database, user_id_star, Request::default()).await.unwrap().starred2;
            assert!(starred.song.is_empty());
            assert!(starred.album.is_empty());
            assert_eq!(
                starred.artist.into_iter().map(|artist| artist.required.id).collect::<Vec<_>>(),
                vec![artist_id]
            );

            let artists: Vec<_> =
                get_artists::handler(database, user_id_star, get_artists::Request::default())
                    .await
                    .unwrap()
                    .artists
                    .index
                    .into_iter()
                    .flat_map(|index| index.artist)
                    .map(|artist| artist.starred.is_some())
                    .collect();
            assert_eq!(artists, vec![true]);
        }

        {
            let starred = handler(database, user_id, Request::default()).await.unwrap().starred2;
            assert!(starred.song.is_empty());
            assert!(starred.album.is_empty());
            assert!(starred.artist.is_empty());

            let artists: Vec<_> =
                get_artists::handler(database, user_id, get_artists::Request::default())
                    .await
                    .unwrap()
                    .artists
                    .index
                    .into_iter()
                    .flat_map(|index| index.artist)
                    .map(|artist| artist.starred.is_some())
                    .collect();
            assert_eq!(artists, vec![false]);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_star_album(
        #[future(awt)]
        #[with(2, 1)]
        mock: Mock,
    ) {
        let database = mock.database();
        let mut music_folder = mock.music_folder(0).await;
        let user_id_star = mock.user_id(0).await;
        let user_id = mock.user_id(1).await;

        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;
        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;
        music_folder
            .add_audio()
            .artists(audio::Artists {
                song: [artist.clone()].into(),
                album: [].into(),
                compilation: false,
            })
            .album(album)
            .call()
            .await;

        star::handler(mock.database(), user_id_star, star::Request {
            album_ids: Some(vec![album_id]),
            ..Default::default()
        })
        .await
        .unwrap();

        {
            let starred =
                handler(database, user_id_star, Request::default()).await.unwrap().starred2;
            assert!(starred.song.is_empty());
            assert!(starred.artist.is_empty());
            assert_eq!(starred.album.into_iter().map(|album| album.id).collect::<Vec<_>>(), vec![
                album_id
            ]);

            let albums: Vec<_> =
                get_artist::handler(database, user_id_star, get_artist::Request { id: artist_id })
                    .await
                    .unwrap()
                    .artist
                    .album
                    .into_iter()
                    .map(|album| album.starred.is_some())
                    .collect();
            assert_eq!(albums, vec![true]);
        }

        {
            let starred = handler(database, user_id, Request::default()).await.unwrap().starred2;
            assert!(starred.song.is_empty());
            assert!(starred.album.is_empty());
            assert!(starred.artist.is_empty());

            let albums: Vec<_> =
                get_artist::handler(database, user_id, get_artist::Request { id: artist_id })
                    .await
                    .unwrap()
                    .artist
                    .album
                    .into_iter()
                    .map(|album| album.starred.is_some())
                    .collect();
            assert_eq!(albums, vec![false]);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_star_song(
        #[future(awt)]
        #[with(2, 1)]
        mock: Mock,
    ) {
        let database = mock.database();
        let mut music_folder = mock.music_folder(0).await;
        let user_id_star = mock.user_id(0).await;
        let user_id = mock.user_id(1).await;

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;
        music_folder.add_audio().album(album).call().await;
        let song_id = *music_folder.database.get_index(0).unwrap().0;

        star::handler(mock.database(), user_id_star, star::Request {
            song_ids: Some(vec![song_id]),
            ..Default::default()
        })
        .await
        .unwrap();

        {
            let starred =
                handler(database, user_id_star, Request::default()).await.unwrap().starred2;
            assert!(starred.album.is_empty());
            assert!(starred.artist.is_empty());
            assert_eq!(
                starred.song.into_iter().map(|song| song.song.id).collect::<Vec<_>>(),
                vec![song_id]
            );

            let songs: Vec<_> =
                get_album::handler(database, user_id_star, get_album::Request { id: album_id })
                    .await
                    .unwrap()
                    .album
                    .song
                    .into_iter()
                    .map(|song| song.song.starred.is_some())
                    .collect();
            assert_eq!(songs, vec![true]);
        }

        {
            let starred = handler(database, user_id, Request::default()).await.unwrap().starred2;
            assert!(starred.song.is_empty());
            assert!(starred.album.is_empty());
            assert!(starred.artist.is_empty());

            let songs: Vec<_> =
                get_album::handler(database, user_id, get_album::Request { id: album_id })
                    .await
                    .unwrap()
                    .album
                    .song
                    .into_iter()
                    .map(|song| song.song.starred.is_some())
                    .collect();
            assert_eq!(songs, vec![false]);
        }
    }
}
