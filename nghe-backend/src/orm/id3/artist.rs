use diesel::dsl::count_distinct;
use diesel::prelude::*;
use uuid::Uuid;

use crate::orm::{albums, artists, songs};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Artist {
    #[diesel(embed)]
    pub required: Required,
    pub index: String,
    #[diesel(select_expression = count_distinct(albums::id.nullable()))]
    pub album_count: i64,
    #[diesel(select_expression = count_distinct(songs::id.nullable()))]
    pub song_count: i64,
    #[diesel(column_name = mbz_id)]
    pub music_brainz_id: Option<Uuid>,
}

pub mod query {
    use diesel::dsl::{auto_type, exists};

    use super::*;
    use crate::orm::{songs_album_artists, songs_artists, user_music_folder_permissions};

    diesel::alias!(albums as albums_sa: AlbumsSA, songs as songs_saa: SongsSAA);

    #[auto_type]
    pub fn artist(user_id: Uuid) -> _ {
        // We will do two joins:
        //
        //  - songs_artists -> songs -> albums.
        //  - use songs for extrating information.
        //  - use albums for checking permission -> use alias `albums_sa`.
        //
        //  - songs_album_artists -> songs -> albums.
        //  - use albums for extracting information and checking permission.
        //  - use songs for joining -> use alias `songs_saa`.
        //
        // Permission should be checked against `albums_sa` and `albums`.
        artists::table
            .left_join(songs_artists::table)
            .left_join(songs::table.on(songs::id.eq(songs_artists::song_id)))
            .left_join(albums_sa.on(albums_sa.field(albums::id).eq(songs::album_id)))
            .left_join(songs_album_artists::table)
            .left_join(songs_saa.on(songs_saa.field(songs::id).eq(songs_album_artists::song_id)))
            .left_join(albums::table.on(albums::id.eq(songs_saa.field(songs::album_id))))
            .group_by(artists::id)
            .order_by((artists::index, artists::name))
            .filter(exists(
                user_music_folder_permissions::table
                    .filter(user_music_folder_permissions::user_id.eq(user_id))
                    .filter(
                        user_music_folder_permissions::music_folder_id
                            .eq(albums_sa.field(albums::music_folder_id))
                            .or(user_music_folder_permissions::music_folder_id
                                .eq(albums::music_folder_id)),
                    ),
            ))
    }
}

#[cfg(test)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_query_artist(
        #[future(awt)] mock: Mock,
        #[values(0, 5)] n_song: i64,
        #[values(0, 6)] n_album: i64,
        #[values(0, 7)] n_both: i64,
    ) {
        let mut music_folder = mock.music_folder(0).await;
        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;

        let mut add_audio_artist =
            async |song: audio::Artist<'static>, album: audio::Artist<'static>, n_song: i64| {
                // Each song will have a different album so `n_song` can be used here.
                music_folder
                    .add_audio()
                    .artists(audio::Artists {
                        song: [song].into(),
                        album: [album].into(),
                        compilation: false,
                    })
                    .n_song(n_song.try_into().unwrap())
                    .call()
                    .await;
            };

        add_audio_artist(artist.clone(), Faker.fake(), n_song).await;
        add_audio_artist(Faker.fake(), artist.clone(), n_album).await;
        add_audio_artist(artist.clone(), artist.clone(), n_both).await;

        let database_artist = query::artist(mock.user(0).await.user.id)
            .filter(artists::id.eq(artist_id))
            .select(Artist::as_select())
            .get_result(&mut mock.get().await)
            .await;

        if n_song == 0 && n_album == 0 && n_both == 0 {
            assert!(database_artist.is_err())
        } else {
            let database_artist = database_artist.unwrap();
            assert_eq!(database_artist.song_count, n_song + n_both);
            assert_eq!(database_artist.album_count, n_album + n_both);
        }
    }
}
