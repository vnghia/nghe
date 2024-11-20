use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;
use o2o::o2o;
use uuid::Uuid;

use super::Song;
use crate::orm::id3::genre;
use crate::Error;

#[derive(Debug, Queryable, Selectable, o2o)]
#[owned_try_into(id3::song::WithAlbumGenres, Error)]
pub struct WithAlbumGenres {
    #[into(~.try_into_api()?)]
    #[diesel(embed)]
    pub song: Song,
    #[diesel(select_expression = sql("any_value(albums.name) album_name"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Text>)]
    pub album: String,
    #[diesel(select_expression = sql("any_value(albums.id) album_id"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Uuid>)]
    pub album_id: Uuid,
    #[into(~.into())]
    #[diesel(embed)]
    pub genres: genre::Genres,
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::id3::song;
    use crate::orm::{albums, genres, permission, songs, songs_genres};

    #[auto_type]
    pub fn unchecked() -> _ {
        let with_album_genres: AsSelect<WithAlbumGenres, crate::orm::Type> =
            WithAlbumGenres::as_select();
        song::query::unchecked_no_group_by()
            .inner_join(albums::table)
            .left_join(songs_genres::table.on(songs_genres::song_id.eq(songs::id)))
            .left_join(genres::table.on(genres::id.eq(songs_genres::genre_id)))
            .group_by(songs::id)
            .select(with_album_genres)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let permission: permission::with_album = permission::with_album(user_id);
        unchecked().filter(permission)
    }
}

#[cfg(test)]
mod test {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::orm::songs;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        let mut music_folder = mock.music_folder(0).await;

        music_folder.add_audio_artist(["1".into(), "2".into()], [Faker.fake()], false, 1).await;
        let song_id = music_folder.song_id(0);

        let database_song = query::with_user_id(mock.user_id(0).await)
            .filter(songs::id.eq(song_id))
            .get_result(&mut mock.get().await)
            .await;

        if allow {
            let database_song = database_song.unwrap();
            let artists: Vec<String> = database_song.song.artists.into();
            assert_eq!(artists, &["1", "2"]);
        } else {
            assert!(database_song.is_err());
        }
    }
}
