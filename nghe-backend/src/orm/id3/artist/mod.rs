pub mod required;
pub mod with_albums;

use diesel::dsl::count_distinct;
use diesel::prelude::*;
use nghe_api::id3;
use nghe_api::id3::builder::artist as builder;
pub use required::Required;
use uuid::Uuid;

use crate::orm::{albums, artists, songs};
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Artist {
    #[diesel(embed)]
    pub required: Required,
    pub index: String,
    #[diesel(select_expression = count_distinct(songs::id.nullable()))]
    pub song_count: i64,
    #[diesel(select_expression = count_distinct(albums::id.nullable()))]
    pub album_count: i64,
    #[diesel(column_name = mbz_id)]
    pub music_brainz_id: Option<Uuid>,
}

pub type BuilderSet =
    builder::SetRoles<builder::SetMusicBrainzId<builder::SetAlbumCount<builder::SetRequired>>>;

impl Artist {
    pub fn try_into_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        let mut roles = vec![];
        if self.song_count > 0 {
            roles.push(id3::artist::Role::Artist);
        }
        if self.album_count > 0 {
            roles.push(id3::artist::Role::AlbumArtist);
        }

        Ok(id3::artist::Artist::builder()
            .required(self.required.into())
            .album_count(self.album_count.try_into()?)
            .music_brainz_id(self.music_brainz_id)
            .roles(roles))
    }
}

impl TryFrom<Artist> for id3::artist::Artist {
    type Error = Error;

    fn try_from(value: Artist) -> Result<Self, Self::Error> {
        Ok(value.try_into_builder()?.build())
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
            .select(artist)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
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
    pub fn with_music_folder<'ids>(user_id: Uuid, music_folder_ids: &'ids [Uuid]) -> _ {
        // Permission should be checked against `albums_sa` and `albums`.
        let with_user_id: with_user_id = with_user_id(user_id);
        with_user_id.filter(
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

        mock.add_audio_artist(
            0,
            [artist.clone()],
            [Faker.fake()],
            false,
            n_song.try_into().unwrap(),
        )
        .await;
        mock.add_audio_artist(
            0,
            [Faker.fake()],
            [artist.clone()],
            false,
            n_album.try_into().unwrap(),
        )
        .await;
        mock.add_audio_artist(
            0,
            [artist.clone()],
            [artist.clone()],
            false,
            n_both.try_into().unwrap(),
        )
        .await;

        let database_artist = query::with_user_id(mock.user_id(0).await)
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
        let music_folder_id_permission = mock.add_music_folder().allow(allow).call().await;
        let music_folder_id = mock.add_music_folder().call().await;

        let user_id = mock.user_id(0).await;
        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;

        let n_both: i64 = (2..4).fake();
        mock.add_audio_artist(
            0,
            [artist.clone()],
            [artist.clone()],
            false,
            n_both.try_into().unwrap(),
        )
        .await;
        mock.add_audio_artist(0, [Faker.fake()], [Faker.fake()], false, (2..4).fake()).await;
        mock.add_audio_artist(1, [Faker.fake()], [Faker.fake()], false, (2..4).fake()).await;

        let database_artists =
            query::with_user_id(user_id).get_results(&mut mock.get().await).await.unwrap();
        assert_eq!(database_artists.len(), if allow { 5 } else { 2 });

        // Only allow music folders will be returned.
        let database_artists_music_folder =
            query::with_music_folder(user_id, &[music_folder_id_permission, music_folder_id])
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

    #[rstest]
    #[case(0, 0, &[])]
    #[case(1, 0, &[id3::artist::Role::Artist])]
    #[case(0, 1, &[id3::artist::Role::AlbumArtist])]
    #[case(1, 1, &[id3::artist::Role::Artist, id3::artist::Role::AlbumArtist])]
    fn test_try_into_api(
        #[case] song_count: i64,
        #[case] album_count: i64,
        #[case] roles: &[id3::artist::Role],
    ) {
        let artist: id3::artist::Artist =
            Artist { song_count, album_count, ..Faker.fake() }.try_into().unwrap();
        assert_eq!(artist.roles, roles);
    }
}
