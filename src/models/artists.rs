pub use crate::schema::artists;
pub use artists::*;

use diesel::prelude::*;
use std::borrow::Cow;

#[derive(Insertable)]
#[diesel(table_name = artists)]
pub struct NewArtist<'a> {
    pub name: Cow<'a, str>,
}

#[cfg(test)]
mod test {
    use super::*;

    use time::OffsetDateTime;
    use uuid::Uuid;

    #[derive(Debug, Queryable, Selectable)]
    #[diesel(table_name = artists)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct Artist {
        pub id: Uuid,
        pub name: String,
        pub index: String,
        pub updated_at: OffsetDateTime,
        pub scanned_at: OffsetDateTime,
    }
}

#[cfg(test)]
pub use test::*;
