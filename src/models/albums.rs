pub use crate::schema::albums;
pub use albums::*;

use diesel::prelude::*;
use std::borrow::Cow;

#[derive(Insertable)]
#[diesel(table_name = albums)]
pub struct NewAlbum<'a> {
    pub name: Cow<'a, str>,
}

#[cfg(test)]
mod test {
    use super::*;

    use time::OffsetDateTime;
    use uuid::Uuid;

    #[derive(Debug, Queryable, Selectable)]
    #[diesel(table_name = albums)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct Album {
        pub id: Uuid,
        pub name: String,
        pub updated_at: OffsetDateTime,
        pub scanned_at: OffsetDateTime,
    }
}

#[cfg(test)]
pub use test::*;
