use std::borrow::Cow;

use uuid::Uuid;

use super::date::Date;

#[derive(Debug)]
pub struct Common<'a> {
    pub name: Cow<'a, str>,
    pub date: Date,
    pub release_date: Date,
    pub original_release_date: Date,
    pub mbz_id: Option<Uuid>,
}
