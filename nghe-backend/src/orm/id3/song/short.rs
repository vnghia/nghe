#![allow(clippy::elidable_lifetime_names)]

use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;
use o2o::o2o;
use uuid::Uuid;

use super::Song;
use crate::Error;
use crate::file::audio;

#[derive(Debug, Queryable, Selectable, o2o)]
#[owned_try_into(id3::song::Short, Error)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
pub struct Short {
    #[into(~.try_into()?)]
    #[diesel(embed)]
    pub song: Song,
    #[diesel(select_expression = sql("any_value(albums.name) album_name"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Text>)]
    pub album: String,
    #[diesel(select_expression = sql("any_value(albums.id) album_id"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Uuid>)]
    pub album_id: Uuid,
}

impl audio::duration::Trait for Short {
    fn duration(&self) -> audio::Duration {
        self.song.duration()
    }
}

pub mod query {
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;
    use crate::orm::id3::song;
    use crate::orm::{albums, permission, songs};

    #[auto_type]
    pub fn with_user_id_unchecked(user_id: Uuid) -> _ {
        let with_user_id_unchecked_no_group_by: song::query::with_user_id_unchecked_no_group_by =
            song::query::with_user_id_unchecked_no_group_by(user_id);
        let full: AsSelect<Short, crate::orm::Type> = Short::as_select();
        with_user_id_unchecked_no_group_by.group_by(songs::id).select(full)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let with_user_id_unchecked: with_user_id_unchecked = with_user_id_unchecked(user_id);
        let permission: permission::with_album = permission::with_album(user_id);
        with_user_id_unchecked.filter(permission)
    }

    #[auto_type]
    pub fn with_music_folder<'ids>(user_id: Uuid, music_folder_ids: &'ids [Uuid]) -> _ {
        let with_user_id: with_user_id = with_user_id(user_id);
        with_user_id.filter(albums::music_folder_id.eq_any(music_folder_ids))
    }
}
