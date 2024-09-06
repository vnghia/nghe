use std::borrow::Cow;

use uuid::Uuid;

use super::date::Date;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Common<'a> {
    pub name: Cow<'a, str>,
    pub date: Date,
    pub release_date: Date,
    pub original_release_date: Date,
    pub mbz_id: Option<Uuid>,
}
