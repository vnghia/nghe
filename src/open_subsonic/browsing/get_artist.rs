use crate::models::*;

use diesel::{
    dsl::{exists, Distinct, Eq, EqAny, Filter, OrFilter, Select},
    ExpressionMethods, QueryDsl,
};
use uuid::Uuid;

pub type SelectDistinctAlbumId = Distinct<Select<songs::table, songs::album_id>>;

pub type SelectDistinctAlbumIdInMusicFolder<'a> =
    Filter<SelectDistinctAlbumId, EqAny<songs::music_folder_id, &'a [Uuid]>>;

pub type SongsAlbumArtistsWithAlbumArtistIdAndSongId<'a> = Filter<
    Filter<songs_album_artists::table, Eq<songs_album_artists::album_artist_id, &'a Uuid>>,
    Eq<songs_album_artists::song_id, songs::id>,
>;

pub type SongsArtistsWithArtistIdAndSongId<'a> = Filter<
    Filter<songs_artists::table, Eq<songs_artists::artist_id, &'a Uuid>>,
    Eq<songs_artists::song_id, songs::id>,
>;

pub type GetArtistOwnALbums<'a> = Filter<
    SelectDistinctAlbumIdInMusicFolder<'a>,
    exists<SongsAlbumArtistsWithAlbumArtistIdAndSongId<'a>>,
>;

pub type GetArtistAlbums<'a> =
    OrFilter<GetArtistOwnALbums<'a>, exists<SongsArtistsWithArtistIdAndSongId<'a>>>;

pub fn get_artist_own_albums_query<'a>(
    music_folder_ids: &'a [Uuid],
    artist_id: &'a Uuid,
) -> GetArtistOwnALbums<'a> {
    songs::table
        .select(songs::album_id)
        .distinct()
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(exists(
            songs_album_artists::table
                .filter(songs_album_artists::album_artist_id.eq(artist_id))
                .filter(songs_album_artists::song_id.eq(songs::id)),
        ))
}

pub fn get_artist_albums_query<'a>(
    music_folder_ids: &'a [Uuid],
    artist_id: &'a Uuid,
) -> GetArtistAlbums<'a> {
    get_artist_own_albums_query(music_folder_ids, artist_id).or_filter(exists(
        songs_artists::table
            .filter(songs_artists::artist_id.eq(artist_id))
            .filter(songs_artists::song_id.eq(songs::id)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::scan::artist::upsert_artists,
        utils::{
            song::tag::SongTag,
            test::{media::song_paths_to_album_ids, setup::setup_users_and_songs},
        },
    };

    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use itertools::Itertools;

    #[tokio::test]
    async fn test_simple_get_artist_own_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, song_fs_info) = setup_users_and_songs(
            0,
            1,
            &[],
            &[n_song],
            (0..n_song)
                .map(|i| {
                    if i < 5 {
                        SongTag {
                            album_artists: vec![artist_name.to_owned()],
                            ..Faker.fake()
                        }
                    } else {
                        Faker.fake()
                    }
                })
                .collect_vec(),
        )
        .await;
        let song_fs_info = song_fs_info
            .into_iter()
            .filter(|(_, v)| v.album_artists.contains(&artist_name.to_owned()))
            .collect();

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(temp_db.pool(), &song_fs_info).await;

        let own_album_ids = get_artist_own_albums_query(&music_folder_ids, &artist_id)
            .get_results::<Uuid>(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(own_album_ids, album_fs_ids);

        let album_ids = get_artist_albums_query(&music_folder_ids, &artist_id)
            .get_results::<Uuid>(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(album_ids, album_fs_ids);
    }

    #[tokio::test]
    async fn test_simple_get_artist_featured_in_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, song_fs_info) = setup_users_and_songs(
            0,
            1,
            &[],
            &[n_song],
            (0..n_song)
                .map(|i| {
                    if i < 5 {
                        SongTag {
                            artists: vec![artist_name.to_owned()],
                            ..Faker.fake()
                        }
                    } else {
                        Faker.fake()
                    }
                })
                .collect_vec(),
        )
        .await;
        let song_fs_info = song_fs_info
            .into_iter()
            .filter(|(_, v)| v.artists.contains(&artist_name.to_owned()))
            .collect();

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(temp_db.pool(), &song_fs_info).await;

        let own_album_ids = get_artist_own_albums_query(&music_folder_ids, &artist_id)
            .get_results::<Uuid>(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap();

        assert!(own_album_ids.is_empty());

        let album_ids = get_artist_albums_query(&music_folder_ids, &artist_id)
            .get_results::<Uuid>(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(album_ids, album_fs_ids);
    }

    #[tokio::test]
    async fn test_simple_get_artist_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, song_fs_info) = setup_users_and_songs(
            0,
            1,
            &[],
            &[n_song],
            (0..n_song)
                .map(|i| {
                    if i < 5 {
                        SongTag {
                            album_artists: vec![artist_name.to_owned()],
                            ..Faker.fake()
                        }
                    } else {
                        SongTag {
                            artists: vec![artist_name.to_owned()],
                            ..Faker.fake()
                        }
                    }
                })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(temp_db.pool(), &song_fs_info).await;

        let own_album_ids = get_artist_own_albums_query(&music_folder_ids, &artist_id)
            .get_results::<Uuid>(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(
            own_album_ids,
            song_paths_to_album_ids(
                temp_db.pool(),
                &song_fs_info
                    .clone()
                    .into_iter()
                    .filter(|(_, v)| v.album_artists.contains(&artist_name.to_owned()))
                    .collect()
            )
            .await
        );

        let album_ids = get_artist_albums_query(&music_folder_ids, &artist_id)
            .get_results::<Uuid>(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(album_ids, album_fs_ids);
    }
}
