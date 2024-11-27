use std::ops::Add;

use diesel::deserialize::FromSql;
use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::{deserialize, sql_types};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Duration {
    pub value: f32,
}

pub type SqlType = sql_types::Record<(sql_types::Uuid, sql_types::Float)>;

#[derive(Debug, Queryable, Selectable)]
pub struct Durations {
    #[diesel(select_expression = sql(
        "array_agg(distinct(songs.id, songs.duration)) song_id_durations"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<SqlType>>)]
    pub value: Vec<Duration>,
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output { value: self.value + rhs.value }
    }
}

impl Durations {
    pub fn count(&self) -> usize {
        self.value.len()
    }
}

impl FromSql<SqlType, crate::orm::Type> for Duration {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let (_, value): (Uuid, f32) = FromSql::<SqlType, crate::orm::Type>::from_sql(bytes)?;
        Ok(Self { value })
    }
}
