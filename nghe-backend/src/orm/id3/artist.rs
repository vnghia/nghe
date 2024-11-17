use diesel::dsl::count_distinct;
use diesel::prelude::*;
use nghe_api::id3;
use uuid::Uuid;

use crate::orm::{albums, artists, songs};
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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

impl Required {
    pub fn into_api(self) -> id3::Artist {
        id3::Artist::builder().id(self.id).name(self.name).build()
    }
}

impl Artist {
    pub fn try_into_api(self) -> Result<id3::Artist, Error> {
        Ok(id3::Artist::builder()
            .id(self.required.id)
            .name(self.required.name)
            .album_count(self.album_count.try_into()?)
            .maybe_music_brainz_id(self.music_brainz_id)
            .build())
    }
}

pub mod query {
    use diesel::dsl::{auto_type, exists, AsSelect};

    use super::*;
    use crate::orm::{songs_album_artists, songs_artists, user_music_folder_permissions};

    diesel::alias!(albums as albums_sa: AlbumsSA, songs as songs_saa: SongsSAA);

    #[auto_type]
    fn unchecked() -> _ {
        // We will do two joins:
        //
        //  - songs_artists -> songs -> albums.
        //  - use songs for extrating information.
        //  - use albums for checking permission -> use alias `albums_sa`.
        //
        //  - songs_album_artists -> songs -> albums.
        //  - use albums for extracting information and checking permission.
        //  - use songs for joining -> use alias `songs_saa`.
        let artist: AsSelect<Artist, crate::orm::Type> = Artist::as_select();
        artists::table
            .left_join(songs_artists::table)
            .left_join(songs::table.on(songs::id.eq(songs_artists::song_id)))
            .left_join(albums_sa.on(albums_sa.field(albums::id).eq(songs::album_id)))
            .left_join(songs_album_artists::table)
            .left_join(songs_saa.on(songs_saa.field(songs::id).eq(songs_album_artists::song_id)))
            .left_join(albums::table.on(albums::id.eq(songs_saa.field(songs::album_id))))
            .group_by(artists::id)
            .order_by((artists::index, artists::name))
            .select(artist)
    }

    #[auto_type]
    pub fn permission(user_id: Uuid) -> _ {
        // Permission should be checked against `albums_sa` and `albums`.
        unchecked().filter(exists(
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

    #[auto_type]
    pub fn music_folder<'ids>(user_id: Uuid, music_folder_ids: &'ids [Uuid]) -> _ {
        // Permission should be checked against `albums_sa` and `albums`.
        let permission: permission = permission(user_id);
        permission.filter(
            albums_sa
                .field(albums::music_folder_id)
                .eq_any(music_folder_ids)
                .or(albums::music_folder_id.eq_any(music_folder_ids)),
        )
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

    async fn add_audio_artist(
        mock: &Mock,
        index: usize,
        song: audio::Artist<'static>,
        album: audio::Artist<'static>,
        n_song: i64,
    ) {
        let mut music_folder = mock.music_folder(index).await;
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
    }

    #[rstest]
    #[tokio::test]
    async fn test_query_artist(
        #[future(awt)] mock: Mock,
        #[values(0, 5)] n_song: i64,
        #[values(0, 6)] n_album: i64,
        #[values(0, 7)] n_both: i64,
    ) {
        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;

        add_audio_artist(&mock, 0, artist.clone(), Faker.fake(), n_song).await;
        add_audio_artist(&mock, 0, Faker.fake(), artist.clone(), n_album).await;
        add_audio_artist(&mock, 0, artist.clone(), artist.clone(), n_both).await;

        let database_artist = query::permission(mock.user(0).await.user.id)
            .filter(artists::id.eq(artist_id))
            .get_result(&mut mock.get().await)
            .await;

        if n_song == 0 && n_album == 0 && n_both == 0 {
            assert!(database_artist.is_err());
        } else {
            let database_artist = database_artist.unwrap();
            assert_eq!(database_artist.song_count, n_song + n_both);
            assert_eq!(database_artist.album_count, n_album + n_both);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_query_artists(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        mock.add_music_folder().call().await;

        let user_id = mock.user(0).await.user.id;
        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;

        let n_both = (2..4).fake();
        add_audio_artist(&mock, 0, artist.clone(), artist.clone(), n_both).await;
        add_audio_artist(&mock, 0, Faker.fake(), Faker.fake(), (2..4).fake()).await;
        add_audio_artist(&mock, 1, Faker.fake(), Faker.fake(), (2..4).fake()).await;

        let database_artists =
            query::permission(user_id).get_results(&mut mock.get().await).await.unwrap();
        assert_eq!(database_artists.len(), if allow { 5 } else { 2 });

        // Only allow music folders will be returned.
        let database_artists_music_folder = query::music_folder(
            user_id,
            &[mock.music_folder(0).await.id(), mock.music_folder(1).await.id()],
        )
        .get_results(&mut mock.get().await)
        .await
        .unwrap();
        assert_eq!(database_artists, database_artists_music_folder);

        let database_artist =
            database_artists.into_iter().find(|artist| artist.required.id == artist_id);

        if allow {
            let database_artist = database_artist.unwrap();
            assert_eq!(database_artist.song_count, n_both);
            assert_eq!(database_artist.album_count, n_both);
        } else {
            assert!(database_artist.is_none());
        }
    }
}
