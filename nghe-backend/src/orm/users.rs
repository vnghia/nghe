use diesel::prelude::*;

use crate::schema::users;

#[derive(Debug, Clone, Copy, Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(super::Type))]
pub struct Role {
    #[diesel(column_name = admin_role)]
    pub admin: bool,
    #[diesel(column_name = stream_role)]
    pub stream: bool,
    #[diesel(column_name = download_role)]
    pub download: bool,
    #[diesel(column_name = share_role)]
    pub share: bool,
}
