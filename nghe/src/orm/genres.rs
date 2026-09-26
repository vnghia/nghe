use std::borrow::Cow;

use diesel::prelude::*;

pub use crate::schema::genres::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = genres, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    pub value: Cow<'a, str>,
}
