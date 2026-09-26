use diesel::deserialize::{self, FromSql};
use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::orm::id3::artist::required::Required;

pub type SqlType = sql_types::Record<(sql_types::Timestamptz, sql_types::Text, sql_types::Uuid)>;

#[derive(Debug, Queryable, Selectable)]
pub struct Artists {
    #[diesel(select_expression = sql(
        "array_agg(distinct (songs_artists.upserted_at, artists.name, artists.id) order by \
        (songs_artists.upserted_at, artists.name, artists.id)) artists"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<SqlType>>)]
    pub value: Vec<Required>,
}

impl From<Artists> for Vec<id3::artist::Required> {
    fn from(value: Artists) -> Self {
        value.value.into_iter().map(Required::into).collect()
    }
}

impl FromSql<SqlType, crate::orm::Type> for Required {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let (_, name, id): (OffsetDateTime, String, Uuid) =
            FromSql::<SqlType, crate::orm::Type>::from_sql(bytes)?;
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
