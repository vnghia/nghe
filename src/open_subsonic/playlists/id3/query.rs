use diesel;
use diesel::dsl::{AsSelect, GroupBy, InnerJoin, Select};
use diesel::{QueryDsl, SelectableHelper};

use super::db::PlaylistId3Db;
use crate::models::*;
use crate::DatabaseType;

pub type GetPlaylistId3Db = Select<
    GroupBy<
        InnerJoin<InnerJoin<playlists_songs::table, playlists::table>, songs::table>,
        playlists::id,
    >,
    AsSelect<PlaylistId3Db, DatabaseType>,
>;

pub fn get_playlist_id3_db() -> GetPlaylistId3Db {
    playlists_songs::table
        .inner_join(playlists::table)
        .inner_join(songs::table)
        .group_by(playlists::id)
        .select(PlaylistId3Db::as_select())
}
