use diesel::prelude::*;
use o2o::o2o;
use uuid::Uuid;

pub use crate::schema::user_keys::{self, *};

#[derive(Insertable)]
#[diesel(table_name = user_keys, check_for_backend(super::Type))]
pub struct New {
    pub user_id: Uuid,
}

#[derive(Queryable, Selectable, o2o)]
#[diesel(table_name = user_keys, check_for_backend(super::Type))]
#[map_owned(nghe_api::auth::ApiKey)]
pub struct Key {
    #[map(api_key)]
    pub id: Uuid,
}
