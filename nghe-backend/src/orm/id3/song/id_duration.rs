use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use num_traits::ToPrimitive;
use uuid::Uuid;

use crate::Error;

#[derive(Debug, Queryable, Selectable)]
pub struct IdDuration {
    #[diesel(select_expression = sql(
        "array_agg(distinct(songs.id, songs.duration)) song_id_durations"
    ))]
    #[diesel(select_expression_type =
        SqlLiteral::<sql_types::Array<sql_types::Record<(sql_types::Uuid, sql_types::Float)>>>
    )]
    pub values: Vec<(Uuid, f32)>,
}

impl IdDuration {
    pub fn song_count(&self) -> usize {
        self.values.len()
    }

    pub fn duration(&self) -> Result<u32, Error> {
        let duration = self.values.iter().map(|(_, duration)| duration).sum::<f32>();
        duration.ceil().to_u32().ok_or_else(|| Error::CouldNotConvertFloatToInteger(duration))
    }
}
