use diesel::prelude::*;
use uuid::Uuid;

pub use crate::schema::user_sessions::{self, *};

#[derive(Insertable)]
#[diesel(table_name = user_sessions, check_for_backend(super::Type))]
pub struct New {
    pub user_id: Uuid,
}
