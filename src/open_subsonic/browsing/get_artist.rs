use crate::models::*;

use diesel::{
    dsl::{exists, Distinct, Eq, EqAny, Filter, OrFilter, Select},
    ExpressionMethods, QueryDsl,
};
use uuid::Uuid;

pub type SelectDistinctAlbumId = Distinct<Select<alias::SongsTable, alias::SongsAlbumId>>;

pub type SelectDistinctAlbumIdInMusicFolder<'a> =
    Filter<SelectDistinctAlbumId, EqAny<alias::SongsMusicFolderId, &'a [Uuid]>>;

pub type SongsAlbumArtistsWithAlbumArtistIdAndSongId<'a> = Filter<
    Filter<alias::SongsAlbumArtistsTable, Eq<alias::SongsAlbumArtistsAlbumArtistId, &'a Uuid>>,
    Eq<alias::SongsAlbumArtistsSongId, alias::SongsId>,
>;

pub type SongsArtistsWithArtistIdAndSongId<'a> = Filter<
    Filter<alias::SongsArtistsTable, Eq<alias::SongsArtistsArtistId, &'a Uuid>>,
    Eq<alias::SongsArtistsSongId, alias::SongsId>,
>;

pub type GetArtistAlbums<'a> = OrFilter<
    Filter<
        SelectDistinctAlbumIdInMusicFolder<'a>,
        exists<SongsAlbumArtistsWithAlbumArtistIdAndSongId<'a>>,
    >,
    exists<SongsArtistsWithArtistIdAndSongId<'a>>,
>;

pub fn get_artist_albums_query<'a>(
    music_folder_ids: &'a [Uuid],
    artist_id: &'a Uuid,
) -> GetArtistAlbums<'a> {
    alias::songs
        .select(alias::songs.field(songs::album_id))
        .distinct()
        .filter(
            alias::songs
                .field(songs::music_folder_id)
                .eq_any(music_folder_ids),
        )
        .filter(exists(
            alias::songs_album_artists
                .filter(
                    alias::songs_album_artists
                        .field(songs_album_artists::album_artist_id)
                        .eq(artist_id),
                )
                .filter(
                    alias::songs_album_artists
                        .field(songs_album_artists::song_id)
                        .eq(alias::songs.field(songs::id)),
                ),
        ))
        .or_filter(exists(
            alias::songs_artists
                .filter(
                    alias::songs_artists
                        .field(songs_artists::artist_id)
                        .eq(artist_id),
                )
                .filter(
                    alias::songs_artists
                        .field(songs_artists::song_id)
                        .eq(alias::songs.field(songs::id)),
                ),
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

        let (temp_db, _, _temp_fs, music_folders, song_fs_info, _) = setup_users_and_songs(
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

        let (temp_db, _, _temp_fs, music_folders, song_fs_info, _) = setup_users_and_songs(
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

        let (temp_db, _, _temp_fs, music_folders, song_fs_info, _) = setup_users_and_songs(
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
