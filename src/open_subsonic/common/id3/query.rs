use diesel::dsl::{AsSelect, Eq, GroupBy, InnerJoin, InnerJoinOn, LeftJoin, Or, Select};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};

use super::db::*;
use crate::models::*;
use crate::DatabaseType;

pub type GetBasicArtistId3Db = Select<
    GroupBy<
        InnerJoinOn<
            LeftJoin<LeftJoin<artists::table, songs_album_artists::table>, songs_artists::table>,
            songs::table,
            Or<Eq<songs::id, songs_album_artists::song_id>, Eq<songs::id, songs_artists::song_id>>,
        >,
        artists::id,
    >,
    AsSelect<BasicArtistId3Db, DatabaseType>,
>;

pub type GetArtistId3Db = Select<GetBasicArtistId3Db, AsSelect<ArtistId3Db, DatabaseType>>;

pub type GetBasicAlbumId3Db = Select<
    GroupBy<InnerJoin<songs::table, albums::table>, albums::id>,
    AsSelect<BasicAlbumId3Db, DatabaseType>,
>;

pub type GetAlbumId3Db = Select<
    LeftJoin<InnerJoin<GetBasicAlbumId3Db, songs_album_artists::table>, GetBasicGenreId3Db>,
    AsSelect<AlbumId3Db, DatabaseType>,
>;

pub type GetBasicSongId3Db = Select<songs::table, AsSelect<BasicSongId3Db, DatabaseType>>;

pub type GetSongId3Db = Select<
    GroupBy<
        LeftJoin<InnerJoin<GetBasicSongId3Db, songs_artists::table>, GetBasicGenreId3Db>,
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
    artists::table
        .left_join(songs_album_artists::table)
        .left_join(songs_artists::table)
        .inner_join(songs::table.on(
            songs::id.eq(songs_album_artists::song_id).or(songs::id.eq(songs_artists::song_id)),
        ))
        .group_by(artists::id)
        .select(BasicArtistId3Db::as_select())
}

pub fn get_artist_id3_db() -> GetArtistId3Db {
    get_basic_artist_id3_db().select(ArtistId3Db::as_select())
}

pub fn get_basic_album_id3_db() -> GetBasicAlbumId3Db {
    songs::table.inner_join(albums::table).group_by(albums::id).select(BasicAlbumId3Db::as_select())
}

pub fn get_album_id3_db() -> GetAlbumId3Db {
    get_basic_album_id3_db()
        .inner_join(songs_album_artists::table)
        .left_join(get_basic_genre_id3_db())
        .select(AlbumId3Db::as_select())
}

pub fn get_basic_song_id3_db() -> GetBasicSongId3Db {
    songs::table.select(BasicSongId3Db::as_select())
}

pub fn get_song_id3_db() -> GetSongId3Db {
    get_basic_song_id3_db()
        .inner_join(songs_artists::table)
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
