use std::borrow::Cow;

pub use albums::*;
use diesel::prelude::*;

pub use crate::schema::albums;

#[derive(Insertable)]
#[diesel(table_name = albums)]
pub struct NewAlbum<'a> {
    pub name: Cow<'a, str>,
}
