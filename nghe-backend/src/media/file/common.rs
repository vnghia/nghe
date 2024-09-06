use std::borrow::Cow;

#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use uuid::Uuid;

use super::date::Date;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy))]
pub struct Common<'a> {
    #[cfg_attr(test, dummy(expr = "Faker.fake::<String>().into()"))]
    pub name: Cow<'a, str>,
    pub date: Date,
    pub release_date: Date,
    pub original_release_date: Date,
    pub mbz_id: Option<Uuid>,
}
