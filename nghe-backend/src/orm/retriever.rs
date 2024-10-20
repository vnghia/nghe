use std::borrow::Cow;

use diesel::dsl::{auto_type, AsSelect};
use diesel::prelude::*;
use diesel::SelectableHelper;
use uuid::Uuid;

use super::{albums, music_folders, permission, songs, user_music_folder_permissions};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
pub struct Song<'mf, 'path> {
    #[diesel(embed)]
    pub music_folder: music_folders::Data<'mf>,
    pub relative_path: Cow<'path, str>,
    #[diesel(embed)]
    pub property: songs::property::File,
}

#[auto_type]
pub fn query<'mf, 'path>(user_id: Uuid, song_id: Uuid) -> _ {
    let permission_query: permission::query = permission::query(user_id);
    let select_song: AsSelect<Song<'mf, 'path>, crate::orm::Type> = Song::<'mf, 'path>::as_select();
    music_folders::table
        .inner_join(user_music_folder_permissions::table)
        .inner_join(albums::table)
        .inner_join(songs::table.on(songs::album_id.eq(albums::id)))
        .filter(songs::id.eq(song_id))
        .select(select_song)
}
