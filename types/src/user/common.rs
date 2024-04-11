use nghe_proc_macros::add_types_derive;
use time::OffsetDateTime;

use super::super::time::time_serde;

#[add_types_derive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Role {
    pub admin_role: bool,
    pub stream_role: bool,
    pub download_role: bool,
    pub share_role: bool,
}

#[derive(Debug)]
#[add_types_derive]
pub struct BasicUser {
    pub username: String,
    #[serde(flatten)]
    pub role: Role,
}

#[derive(Debug)]
#[add_types_derive]
pub struct User {
    #[serde(flatten)]
    pub basic: BasicUser,
    #[serde(with = "time_serde::iso8601_datetime")]
    pub created_at: OffsetDateTime,
}
