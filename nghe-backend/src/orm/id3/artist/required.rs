use diesel::deserialize::{self, FromSql};
use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;
use o2o::o2o;
use uuid::Uuid;

use crate::orm::artists;

#[derive(Debug, Queryable, Selectable, o2o)]
#[owned_into(id3::artist::Required)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}

pub type SqlType = sql_types::Record<(sql_types::Text, sql_types::Uuid)>;

#[derive(Debug, Queryable, Selectable)]
pub struct Artists {
    #[diesel(select_expression = sql(
        "array_agg(distinct (artists.name, artists.id) order by (artists.name, artists.id)) artists"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<SqlType>>)]
    pub value: Vec<Required>,
}

impl From<Artists> for Vec<id3::artist::Required> {
    fn from(value: Artists) -> Self {
        value.value.into_iter().map(Required::into).collect()
    }
}

pub mod query {
    use diesel::dsl::auto_type;

    use super::*;
    use crate::orm::{songs_album_artists, songs_artists};

    #[auto_type]
    pub fn album() -> _ {
        artists::table.on(artists::id.eq(songs_album_artists::album_artist_id))
    }

    #[auto_type]
    pub fn song() -> _ {
        artists::table.on(artists::id.eq(songs_artists::artist_id))
    }
}

impl FromSql<SqlType, crate::orm::Type> for Required {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let (name, id) = FromSql::<SqlType, crate::orm::Type>::from_sql(bytes)?;
        Ok(Self { id, name })
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    impl From<Artists> for Vec<String> {
        fn from(value: Artists) -> Self {
            value.value.into_iter().map(|artist| artist.name).collect()
        }
    }
}
