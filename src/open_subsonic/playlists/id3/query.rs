use diesel::dsl::{AsSelect, GroupBy, InnerJoin, Select};
use diesel::{self, QueryDsl, SelectableHelper};

use super::{PlaylistId3Db, PlaylistId3WithSongIdsDb};
use crate::models::*;
use crate::DatabaseType;

pub type GetPlaylistId3Db = Select<
    GroupBy<
        InnerJoin<InnerJoin<playlists_songs::table, playlists::table>, songs::table>,
        playlists::id,
    >,
    AsSelect<PlaylistId3Db, DatabaseType>,
>;

pub type GetPlaylistId3WithSongIdsDb =
    Select<GetPlaylistId3Db, AsSelect<PlaylistId3WithSongIdsDb, DatabaseType>>;

pub fn get_playlist_id3_db() -> GetPlaylistId3Db {
    playlists_songs::table
        .inner_join(playlists::table)
        .inner_join(songs::table)
        .group_by(playlists::id)
        .select(PlaylistId3Db::as_select())
}

pub fn get_playlist_id3_with_song_ids_db() -> GetPlaylistId3WithSongIdsDb {
    get_playlist_id3_db().select(PlaylistId3WithSongIdsDb::as_select())
}
