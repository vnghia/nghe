use diesel::deserialize::FromSql;
use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::{deserialize, sql_types};
use uuid::Uuid;

use crate::file::audio;

pub type SqlType = sql_types::Record<(sql_types::Uuid, sql_types::Float)>;

#[derive(Debug, Queryable, Selectable)]
pub struct Durations {
    #[diesel(select_expression = sql(
        "array_agg(distinct(songs.id, songs.duration)) \
        filter (where songs.id is not null) song_id_durations"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Nullable<sql_types::Array<SqlType>>>)]
    pub value: Option<Vec<audio::Duration>>,
}

impl Durations {
    pub fn count(&self) -> usize {
        self.value.as_ref().map(Vec::len).unwrap_or_default()
    }
}

impl FromSql<SqlType, crate::orm::Type> for audio::Duration {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let (_, value): (Uuid, f32) = FromSql::<SqlType, crate::orm::Type>::from_sql(bytes)?;
        Ok(value.into())
    }
}

impl audio::duration::Trait for Durations {
    fn duration(&self) -> audio::Duration {
        self.value.as_ref().map(Vec::duration).unwrap_or_default()
    }
}
