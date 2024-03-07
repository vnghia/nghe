pub use crate::schema::albums;
pub use albums::*;

use diesel::prelude::*;
use std::borrow::Cow;

#[derive(Insertable)]
#[diesel(table_name = albums)]
pub struct NewAlbum<'a> {
    pub name: Cow<'a, str>,
}
