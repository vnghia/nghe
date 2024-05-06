use diesel::dsl::{
    not, And, AsSelect, Eq, Filter, GroupBy, InnerJoin, InnerJoinOn, IsNull, LeftJoin, LeftJoinOn,
    Select,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};

use super::db::*;
use crate::models::*;
use crate::DatabaseType;

pub type GetBasicArtistId3Db = Select<artists::table, AsSelect<BasicArtistId3Db, DatabaseType>>;

pub type GetAlbumArtistId3Db = Select<
    GroupBy<
        InnerJoinOn<
            InnerJoin<GetBasicArtistId3Db, songs_album_artists::table>,
            songs::table,
            Eq<songs::id, songs_album_artists::song_id>,
        >,
        artists::id,
    >,
    AsSelect<ArtistAlbumCountId3Db, DatabaseType>,
>;

pub type GetNoCompilationAlbumArtistId3Db =
    Filter<GetAlbumArtistId3Db, not<songs_album_artists::compilation>>;

pub type GetSongArtistId3Db = Select<
    GroupBy<
        InnerJoinOn<
            InnerJoin<GetBasicArtistId3Db, songs_artists::table>,
            songs::table,
            Eq<songs::id, songs_artists::song_id>,
        >,
        artists::id,
    >,
    AsSelect<ArtistId3Db, DatabaseType>,
>;

pub type GetNoAlbumArtistId3Db = Filter<
    LeftJoinOn<
        GetSongArtistId3Db,
        songs_album_artists::table,
        And<
            Eq<songs_album_artists::album_artist_id, artists::id>,
            Eq<songs_album_artists::song_id, songs::id>,
        >,
    >,
    IsNull<songs_album_artists::album_artist_id>,
>;

pub type GetBasicAlbumId3Db = Select<
    GroupBy<InnerJoin<songs::table, albums::table>, albums::id>,
    AsSelect<BasicAlbumId3Db, DatabaseType>,
>;

pub type GetAlbumId3Db = Select<
    Filter<
        LeftJoin<InnerJoin<GetBasicAlbumId3Db, songs_album_artists::table>, GetBasicGenreId3Db>,
        not<songs_album_artists::compilation>,
    >,
    AsSelect<AlbumId3Db, DatabaseType>,
>;

pub type GetBasicSongId3Db = Select<songs::table, AsSelect<BasicSongId3Db, DatabaseType>>;

pub type GetSongId3Db = Select<
    GroupBy<
        LeftJoin<
            InnerJoin<InnerJoin<GetBasicSongId3Db, songs_artists::table>, albums::table>,
            GetBasicGenreId3Db,
        >,
        songs::id,
    >,
    AsSelect<SongId3Db, DatabaseType>,
>;

pub type GetBasicGenreId3Db = InnerJoin<songs_genres::table, genres::table>;

pub type GetGenreId3Db = Select<
    GroupBy<InnerJoin<GetBasicGenreId3Db, songs::table>, genres::id>,
    AsSelect<GenreId3Db, DatabaseType>,
>;

pub type GetLyricId3Db =
    Select<InnerJoin<lyrics::table, songs::table>, AsSelect<LyricId3Db, DatabaseType>>;

pub fn get_basic_artist_id3_db() -> GetBasicArtistId3Db {
    artists::table.select(BasicArtistId3Db::as_select())
}

pub fn get_album_artist_id3_db() -> GetAlbumArtistId3Db {
    get_basic_artist_id3_db()
        .inner_join(songs_album_artists::table)
        .inner_join(songs::table.on(songs::id.eq(songs_album_artists::song_id)))
        .group_by(artists::id)
        .select(ArtistAlbumCountId3Db::as_select())
}

pub fn get_no_compilation_album_artist_id3_db() -> GetNoCompilationAlbumArtistId3Db {
    get_album_artist_id3_db().filter(not(songs_album_artists::compilation))
}

pub fn get_song_artist_id3_db() -> GetSongArtistId3Db {
    get_basic_artist_id3_db()
        .inner_join(songs_artists::table)
        .inner_join(songs::table.on(songs::id.eq(songs_artists::song_id)))
        .group_by(artists::id)
        .select(ArtistId3Db::as_select())
}

pub fn get_no_album_artist_id3_db() -> GetNoAlbumArtistId3Db {
    get_song_artist_id3_db()
        .left_join(
            songs_album_artists::table.on(songs_album_artists::album_artist_id
                .eq(artists::id)
                .and(songs_album_artists::song_id.eq(songs::id))),
        )
        .filter(songs_album_artists::album_artist_id.is_null())
}

pub fn get_basic_album_id3_db() -> GetBasicAlbumId3Db {
    songs::table.inner_join(albums::table).group_by(albums::id).select(BasicAlbumId3Db::as_select())
}

pub fn get_album_id3_db() -> GetAlbumId3Db {
    get_basic_album_id3_db()
        .inner_join(songs_album_artists::table)
        .left_join(get_basic_genre_id3_db())
        .filter(not(songs_album_artists::compilation))
        .select(AlbumId3Db::as_select())
}

pub fn get_basic_song_id3_db() -> GetBasicSongId3Db {
    songs::table.select(BasicSongId3Db::as_select())
}

pub fn get_song_id3_db() -> GetSongId3Db {
    get_basic_song_id3_db()
        .inner_join(songs_artists::table)
        .inner_join(albums::table)
        .left_join(get_basic_genre_id3_db())
        .group_by(songs::id)
        .select(SongId3Db::as_select())
}

pub fn get_basic_genre_id3_db() -> GetBasicGenreId3Db {
    songs_genres::table.inner_join(genres::table)
}

pub fn get_genre_id3_db() -> GetGenreId3Db {
    get_basic_genre_id3_db()
        .inner_join(songs::table)
        .group_by(genres::id)
        .select(GenreId3Db::as_select())
}

pub fn get_lyric_id3_db() -> GetLyricId3Db {
    lyrics::table.inner_join(songs::table).select(LyricId3Db::as_select())
}

#[cfg(test)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use itertools::Itertools;

    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_album_artist_id3_db() {
        let mut infra = Infra::new().await.add_user(None).await.n_folder(1).await;
        infra
            .add_songs(
                0,
                vec![
                    SongTag {
                        artists: vec!["artist-no-album-1".into(), "artist-no-album-2".into()],
                        album_artists: vec!["artist-album-1".into(), "artist-album-2".into()],
                        compilation: false,
                        ..Faker.fake()
                    },
                    SongTag {
                        artists: vec!["artist-no-album-3".into(), "artist-album-2".into()],
                        album_artists: vec!["artist-album-2".into()],
                        compilation: false,
                        ..Faker.fake()
                    },
                ],
            )
            .await
            .scan(.., None)
            .await;
        let artists = get_album_artist_id3_db()
            .get_results(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| a.artist.basic.no_id.name.cmp(&b.artist.basic.no_id.name))
            .collect_vec();
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].artist.basic.no_id.name, "artist-album-1");
        assert_eq!(artists[0].album_count, 1);
        assert_eq!(artists[1].artist.basic.no_id.name, "artist-album-2");
        assert_eq!(artists[1].album_count, 2);
    }

    #[tokio::test]
    async fn test_get_album_artist_id3_db_compilation() {
        let mut infra = Infra::new().await.add_user(None).await.n_folder(1).await;
        infra
            .add_songs(
                0,
                vec![
                    SongTag {
                        artists: vec!["artist-no-album-1".into(), "artist-no-album-2".into()],
                        album_artists: vec!["artist-album-1".into(), "artist-album-2".into()],
                        compilation: true,
                        ..Faker.fake()
                    },
                    SongTag {
                        artists: vec!["artist-no-album-3".into(), "artist-album-2".into()],
                        album_artists: vec!["artist-album-2".into()],
                        compilation: false,
                        ..Faker.fake()
                    },
                ],
            )
            .await
            .scan(.., None)
            .await;
        let artists = get_album_artist_id3_db()
            .get_results(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| a.artist.basic.no_id.name.cmp(&b.artist.basic.no_id.name))
            .collect_vec();
        assert_eq!(artists.len(), 4);
        assert_eq!(artists[0].artist.basic.no_id.name, "artist-album-1");
        assert_eq!(artists[0].album_count, 1);
        assert_eq!(artists[1].artist.basic.no_id.name, "artist-album-2");
        assert_eq!(artists[1].album_count, 2);
        assert_eq!(artists[2].artist.basic.no_id.name, "artist-no-album-1");
        assert_eq!(artists[2].album_count, 1);
        assert_eq!(artists[3].artist.basic.no_id.name, "artist-no-album-2");
        assert_eq!(artists[3].album_count, 1);

        let artists = get_no_compilation_album_artist_id3_db()
            .get_results(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| a.artist.basic.no_id.name.cmp(&b.artist.basic.no_id.name))
            .collect_vec();
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].artist.basic.no_id.name, "artist-album-1");
        assert_eq!(artists[0].album_count, 1);
        assert_eq!(artists[1].artist.basic.no_id.name, "artist-album-2");
        assert_eq!(artists[1].album_count, 2);
    }

    #[tokio::test]
    async fn test_get_song_artist_id3_db() {
        let mut infra = Infra::new().await.add_user(None).await.n_folder(1).await;
        infra
            .add_songs(
                0,
                vec![
                    SongTag {
                        artists: vec!["artist-no-album-1".into(), "artist-no-album-2".into()],
                        album_artists: vec!["artist-album-1".into(), "artist-album-2".into()],
                        compilation: false,
                        ..Faker.fake()
                    },
                    SongTag {
                        artists: vec!["artist-no-album-1".into(), "artist-album-2".into()],
                        album_artists: vec!["artist-album-2".into()],
                        compilation: false,
                        ..Faker.fake()
                    },
                ],
            )
            .await
            .scan(.., None)
            .await;

        let artists = get_song_artist_id3_db()
            .get_results(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| a.basic.no_id.name.cmp(&b.basic.no_id.name))
            .collect_vec();
        assert_eq!(artists.len(), 3);
        assert_eq!(artists[0].basic.no_id.name, "artist-album-2");
        assert_eq!(artists[1].basic.no_id.name, "artist-no-album-1");
        assert_eq!(artists[2].basic.no_id.name, "artist-no-album-2");

        let artists = get_no_album_artist_id3_db()
            .get_results(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| a.basic.no_id.name.cmp(&b.basic.no_id.name))
            .collect_vec();
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].basic.no_id.name, "artist-no-album-1");
        assert_eq!(artists[1].basic.no_id.name, "artist-no-album-2");
    }
}
