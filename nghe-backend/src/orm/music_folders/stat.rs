use std::borrow::Cow;

use diesel::dsl::{Nullable, count_distinct};
use diesel::prelude::*;
use o2o::o2o;

use super::{FilesystemType, music_folders};
use crate::orm::{albums, songs};

diesel::alias!(songs as songs_total_size: SongsTotalSize);

#[derive(Debug, Queryable, Selectable, o2o)]
#[diesel(table_name = music_folders, check_for_backend(crate::orm::Type))]
#[owned_into(nghe_api::music_folder::get::Response)]
pub struct Stat<'a> {
    #[into(~.into_owned())]
    pub name: Cow<'a, str>,
    #[into(~.into_owned())]
    pub path: Cow<'a, str>,
    #[diesel(column_name = fs_type)]
    #[into(~.into())]
    pub ty: FilesystemType,
    #[diesel(select_expression = count_distinct(albums::id.nullable()))]
    #[diesel(select_expression_type = count_distinct<Nullable<albums::id>>)]
    #[into(~.cast_unsigned())]
    pub album_count: i64,
    #[diesel(select_expression = count_distinct(songs::id.nullable()))]
    #[diesel(select_expression_type = count_distinct<Nullable<songs::id>>)]
    #[into(~.cast_unsigned())]
    pub song_count: i64,
}

pub mod query {
    use diesel::QueryDsl;
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;
    use crate::orm::{albums, music_folders};

    #[auto_type]
    pub fn unchecked() -> _ {
        let stat: AsSelect<Stat<'static>, crate::orm::Type> = Stat::as_select();
        music_folders::table
            .left_join(albums::table)
            .left_join(songs::table.on(songs::album_id.eq(albums::id)))
            .group_by(music_folders::id)
            .select(stat)
    }
}
