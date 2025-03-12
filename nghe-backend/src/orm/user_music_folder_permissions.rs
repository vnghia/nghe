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

mod check {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Permission, user_music_folder_permissions};
    use crate::Error;
    use crate::database::Database;
    use crate::orm::users;

    impl Permission {
        pub async fn query(
            database: &Database,
            user_id: Uuid,
            music_folder_id: Uuid,
        ) -> Result<Self, Error> {
            user_music_folder_permissions::table
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(user_music_folder_permissions::music_folder_id.eq(music_folder_id))
                .select(Self::as_select())
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }

        pub async fn check_owner(
            database: &Database,
            user_id: Uuid,
            music_folder_id: Uuid,
        ) -> Result<(), Error> {
            let permission = Self::query(database, user_id, music_folder_id).await?;
            if permission.owner {
                Ok(())
            } else {
                users::Role::check_admin(database, user_id).await
            }
        }
    }
}
