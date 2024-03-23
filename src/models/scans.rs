use std::borrow::Cow;

use diesel::prelude::*;
pub use scans::*;
use time::OffsetDateTime;

pub use crate::schema::scans;

#[derive(AsChangeset)]
#[diesel(table_name = scans)]
pub struct FinishScan<'a> {
    pub is_scanning: bool,
    pub finished_at: OffsetDateTime,
    pub scanned_count: i64,
    pub error_message: Option<Cow<'a, str>>,
}
