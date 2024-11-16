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
    async fn test_query_song_artist_only(#[future(awt)] mock: Mock) {
        let database = mock.database();
        let prefixes = &mock.config.index.ignore_prefixes;

        let mut music_folder = mock.music_folder(0).await;
        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert(database, prefixes).await.unwrap();
        let n_song: i64 = (2..4).fake();
        music_folder
            .add_audio()
            .artists(audio::Artists {
                song: [artist].into(),
                album: [Faker.fake()].into(),
                compilation: false,
            })
            .n_song(n_song.try_into().unwrap())
            .call()
            .await;

        let database_artist = query::artist(mock.user(0).await.user.id)
            .filter(artists::id.eq(artist_id))
            .select(Artist::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        assert_eq!(database_artist.song_count, n_song);
        assert_eq!(database_artist.album_count, 0);
    }
}
