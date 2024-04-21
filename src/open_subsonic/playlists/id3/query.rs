use diesel::dsl::{AsSelect, Eq, Filter, GroupBy, IsNull, LeftJoin, LeftJoinOn, Or, Select};
use diesel::{
    self, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper,
};
use uuid::Uuid;

use super::{PlaylistId3Db, PlaylistId3WithSongIdsDb};
use crate::models::*;
use crate::open_subsonic::permission::{with_permission, WithPermission};
use crate::DatabaseType;

pub type GetPlaylistId3Db = Select<
    GroupBy<
        Filter<
            LeftJoinOn<
                LeftJoin<playlists::table, playlists_songs::table>,
                songs::table,
                Eq<songs::id, playlists_songs::song_id>,
            >,
            Or<IsNull<songs::id>, WithPermission>,
        >,
        playlists::id,
    >,
    AsSelect<PlaylistId3Db, DatabaseType>,
>;

pub type GetPlaylistId3WithSongIdsDb =
    Select<GetPlaylistId3Db, AsSelect<PlaylistId3WithSongIdsDb, DatabaseType>>;

pub fn get_playlist_id3_db(user_id: Uuid) -> GetPlaylistId3Db {
    // null song id means the current playlist has no song.
    playlists::table
        .left_join(playlists_songs::table)
        .left_join(songs::table.on(songs::id.eq(playlists_songs::song_id)))
        .filter(songs::id.is_null().or(with_permission(user_id)))
        .group_by(playlists::id)
        .select(PlaylistId3Db::as_select())
}

pub fn get_playlist_id3_with_song_ids_db(user_id: Uuid) -> GetPlaylistId3WithSongIdsDb {
    get_playlist_id3_db(user_id).select(PlaylistId3WithSongIdsDb::as_select())
}
