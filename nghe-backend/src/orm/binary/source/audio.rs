use std::borrow::Cow;

use diesel::dsl::{auto_type, AsSelect};
use diesel::prelude::*;
use diesel::SelectableHelper;
use uuid::Uuid;

use crate::orm::{albums, music_folders, permission, songs};

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
    let permission: permission::with_album = permission::with_album(user_id);
    let select_song: AsSelect<Song<'mf, 'path>, crate::orm::Type> = Song::as_select();
    albums::table
        .inner_join(songs::table)
        .inner_join(music_folders::table)
        .filter(songs::id.eq(song_id))
        .filter(permission)
        .select(select_song)
}
