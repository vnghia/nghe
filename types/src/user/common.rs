use nghe_proc_macros::{add_role_fields, add_types_derive};
use time::OffsetDateTime;

use super::super::time::time_serde;

#[add_role_fields]
#[add_types_derive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Role {}

#[derive(Debug, Clone)]
#[add_types_derive]
pub struct BasicUser {
    pub username: String,
    #[serde(flatten)]
    pub role: Role,
}

#[derive(Debug, Clone)]
#[add_types_derive]
pub struct User {
    #[serde(flatten)]
    pub basic: BasicUser,
    #[serde(with = "time_serde::iso8601_datetime")]
    pub created_at: OffsetDateTime,
}
