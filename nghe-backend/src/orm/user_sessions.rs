use diesel::prelude::*;
use o2o::o2o;
use uuid::Uuid;

pub use crate::schema::user_sessions::{self, *};

#[derive(Insertable)]
#[diesel(table_name = user_sessions, check_for_backend(super::Type))]
pub struct New {
    pub user_id: Uuid,
}

#[derive(Queryable, Selectable, o2o)]
#[diesel(table_name = user_sessions, check_for_backend(super::Type))]
#[map_owned(nghe_api::auth::ApiKey)]
pub struct ApiKey {
    #[diesel(column_name = id)]
    pub api_key: Uuid,
}
