use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::id3;

#[derive(Debug, Queryable, Selectable)]
pub struct Genres {
    #[diesel(select_expression = sql(
        "array_remove(array_agg(distinct genres.value order by genres.value), null) genres"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Text>>)]
    pub value: Vec<String>,
}

impl From<Genres> for id3::genre::Genres {
    fn from(value: Genres) -> Self {
        value.value.into_iter().collect()
    }
}
