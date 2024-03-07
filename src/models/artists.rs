pub use crate::schema::artists;
pub use artists::*;

use diesel::prelude::*;
use std::borrow::Cow;

#[derive(Insertable)]
#[diesel(table_name = artists)]
pub struct NewArtist<'a> {
    pub name: Cow<'a, str>,
}
