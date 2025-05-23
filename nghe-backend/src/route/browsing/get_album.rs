use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::browsing::get_album::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{albums, id3};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(Response {
        album: id3::album::full::query::with_user_id(user_id)
            .filter(albums::id.eq(request.id))
            .get_result(&mut database.get().await?)
            .await?
            .try_into(database, user_id)
            .await?,
    })
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::file::audio::duration::Trait as _;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_sorted(#[future(awt)] mock: Mock, #[values(true, false)] compilation: bool) {
        let mut music_folder = mock.music_folder(0).await;

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;

        let n_song = (2..4).fake();
        for i in 0..n_song {
            music_folder
                .add_audio()
                .album(album.clone())
                .artists(audio::Artists {
                    song: [i.to_string().into()].into(),
                    album: [(i + 1).to_string().into()].into(),
                    compilation,
                })
                .song(audio::Song {
                    track_disc: audio::TrackDisc {
                        track: audio::position::Position { number: Some(i + 1), ..Faker.fake() },
                        disc: audio::position::Position { number: None, ..Faker.fake() },
                    },
                    ..Faker.fake()
                })
                .call()
                .await;
        }

        let album = handler(mock.database(), mock.user_id(0).await, Request { id: album_id })
            .await
            .unwrap()
            .album;

        let artists: Vec<_> = album.artists.into_iter().map(|artist| artist.name).collect();
        let expected_artists: Vec<_> = (0..n_song).map(|i| (i + 1).to_string()).collect();
        assert_eq!(artists, expected_artists);
        assert_eq!(album.is_compilation, compilation);

        assert_eq!(album.album.duration, music_folder.database.duration().0);

        let n_song: usize = n_song.into();
        assert_eq!(album.song.len(), n_song);
        for (i, song) in album.song.into_iter().enumerate() {
            let track: u16 = (i + 1).try_into().unwrap();
            assert_eq!(song.song.track.unwrap(), track);
        }
    }
}
