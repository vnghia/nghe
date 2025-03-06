#![allow(clippy::elidable_lifetime_names)]

use diesel::prelude::*;
use nghe_api::id3;
use o2o::o2o;
use uuid::Uuid;

use super::short::Short;
use crate::Error;
use crate::orm::id3::genre;

#[derive(Debug, Queryable, Selectable, o2o)]
#[owned_try_into(id3::song::Full, Error)]
pub struct Full {
    #[into(~.try_into()?)]
    #[diesel(embed)]
    pub short: Short,
    #[into(~.into())]
    #[diesel(embed)]
    pub genres: genre::Genres,
}

pub mod query {
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;
    use crate::orm::id3::song;
    use crate::orm::{albums, genres, permission, songs, songs_genres};

    #[auto_type]
    pub fn with_user_id_unchecked(user_id: Uuid) -> _ {
        let with_user_id_unchecked_no_group_by: song::query::with_user_id_unchecked_no_group_by =
            song::query::with_user_id_unchecked_no_group_by(user_id);
        let full: AsSelect<Full, crate::orm::Type> = Full::as_select();
        with_user_id_unchecked_no_group_by
            .left_join(songs_genres::table.on(songs_genres::song_id.eq(songs::id)))
            .left_join(genres::table.on(genres::id.eq(songs_genres::genre_id)))
            .group_by(songs::id)
            .select(full)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let with_user_id_unchecked: with_user_id_unchecked = with_user_id_unchecked(user_id);
        let permission: permission::with_album = permission::with_album(user_id);
        with_user_id_unchecked.filter(permission)
    }

    #[auto_type]
    pub fn with_music_folder<'ids>(user_id: Uuid, music_folder_ids: &'ids [Uuid]) -> _ {
        let with_user_id: with_user_id = with_user_id(user_id);
        with_user_id.filter(albums::music_folder_id.eq_any(music_folder_ids))
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::orm::songs;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
        #[values(0, 2)] n_genre: usize,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        let mut music_folder = mock.music_folder(0).await;

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;
        let artists: Vec<_> = (0..(2..4).fake()).map(|i| i.to_string()).collect();
        music_folder
            .add_audio()
            .album(album.clone())
            .artists(audio::Artists {
                song: artists.clone().into_iter().map(String::into).collect(),
                album: [Faker.fake()].into(),
                compilation: false,
            })
            .genres(fake::vec![String; n_genre].into_iter().collect())
            .call()
            .await;
        let song_id = music_folder.song_id(0);

        let database_song = query::with_user_id(mock.user_id(0).await)
            .filter(songs::id.eq(song_id))
            .get_result(&mut mock.get().await)
            .await;

        if allow {
            let database_song = database_song.unwrap();
            let database_artists: Vec<String> = database_song.short.song.artists.into();
            assert_eq!(database_song.short.album, album.name);
            assert_eq!(database_song.short.album_id, album_id);
            assert_eq!(database_artists, artists);
            assert_eq!(database_song.genres.value.len(), n_genre);
        } else {
            assert!(database_song.is_err());
        }
    }
}
