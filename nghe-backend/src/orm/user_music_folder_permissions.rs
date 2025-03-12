use diesel::prelude::*;
use o2o::o2o;
use uuid::Uuid;

pub use crate::schema::user_music_folder_permissions::{self, *};

#[derive(Debug, Clone, Copy, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[diesel(table_name = user_music_folder_permissions, check_for_backend(crate::orm::Type))]
#[map_owned(nghe_api::permission::Permission)]
#[cfg_attr(test, derive(Default, PartialEq, Eq))]
pub struct Permission {
    pub owner: bool,
    pub share: bool,
}

#[derive(Insertable)]
#[diesel(table_name = user_music_folder_permissions, check_for_backend(super::Type))]
pub struct New {
    pub user_id: Uuid,
    pub music_folder_id: Uuid,
    #[diesel(embed)]
    pub permission: Permission,
}
