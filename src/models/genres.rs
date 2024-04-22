use std::borrow::Cow;

use diesel::prelude::*;
pub use genres::*;

pub use crate::schema::genres;

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = genres)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, PartialEq, Eq, PartialOrd, Ord))]
pub struct NewGenre<'a> {
    pub value: Cow<'a, str>,
}

pub type Genre = NewGenre<'static>;

impl From<String> for Genre {
    fn from(v: String) -> Self {
        Self { value: v.into() }
    }
}

impl From<&str> for Genre {
    fn from(v: &str) -> Self {
        v.to_string().into()
    }
}
