use std::borrow::Cow;

pub use artists::*;
use diesel::prelude::*;

pub use crate::schema::artists;

#[derive(Insertable)]
#[diesel(table_name = artists)]
pub struct NewArtist<'a> {
    pub name: Cow<'a, str>,
}
