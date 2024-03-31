use diesel::dsl::{
    count_distinct, AsSelect, Eq, GroupBy, Gt, Having, InnerJoin, InnerJoinOn, LeftJoin, Or, Select,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};

use super::db::*;
use crate::models::*;
use crate::DatabaseType;

pub type GetBasicArtistId3Db = Select<
    Having<
        GroupBy<
            InnerJoinOn<
                LeftJoin<
                    LeftJoin<artists::table, songs_album_artists::table>,
                    songs_artists::table,
                >,
                songs::table,
                Or<
                    Eq<songs::id, songs_album_artists::song_id>,
                    Eq<songs::id, songs_artists::song_id>,
                >,
            >,
            artists::id,
        >,
        Gt<count_distinct<songs::album_id>, i64>,
    >,
    AsSelect<BasicArtistId3Db, DatabaseType>,
>;

pub type GetArtistId3Db = Select<GetBasicArtistId3Db, AsSelect<ArtistId3Db, DatabaseType>>;

pub type GetBasicAlbumId3Db = Select<
    GroupBy<InnerJoin<songs::table, albums::table>, albums::id>,
    AsSelect<BasicAlbumId3Db, DatabaseType>,
>;

pub type GetAlbumId3Db = Select<
    Having<
        InnerJoin<GetBasicAlbumId3Db, songs_album_artists::table>,
        Gt<count_distinct<songs::id>, i64>,
    >,
    AsSelect<AlbumId3Db, DatabaseType>,
>;

pub type GetBasicSongId3Db = Select<songs::table, AsSelect<BasicSongId3Db, DatabaseType>>;

pub type GetSongId3Db = Select<
    GroupBy<InnerJoin<GetBasicSongId3Db, songs_artists::table>, songs::id>,
    AsSelect<SongId3Db, DatabaseType>,
>;

pub fn get_basic_artist_id3_db() -> GetBasicArtistId3Db {
    artists::table
        .left_join(songs_album_artists::table)
        .left_join(songs_artists::table)
        .inner_join(songs::table.on(
            songs::id.eq(songs_album_artists::song_id).or(songs::id.eq(songs_artists::song_id)),
        ))
        .group_by(artists::id)
        .having(count_distinct(songs::album_id).gt(0))
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
        .having(count_distinct(songs::id).gt(0))
        .select(AlbumId3Db::as_select())
}

pub fn get_basic_song_id3_db() -> GetBasicSongId3Db {
    songs::table.select(BasicSongId3Db::as_select())
}

pub fn get_song_id3_db() -> GetSongId3Db {
    get_basic_song_id3_db()
        .inner_join(songs_artists::table)
        .group_by(songs::id)
        .select(SongId3Db::as_select())
}
