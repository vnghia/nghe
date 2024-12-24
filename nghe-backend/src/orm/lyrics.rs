use std::borrow::Cow;

use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::lyrics::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = lyrics, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    pub language: Cow<'a, str>,
    #[diesel(select_expression = sql("lyrics.line_starts line_starts"))]
    #[diesel(select_expression_type =
        SqlLiteral<sql_types::Nullable<sql_types::Array<sql_types::Integer>>>
    )]
    pub line_starts: Option<Vec<i32>>,
    #[diesel(select_expression = sql("lyrics.line_values line_values"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Array<sql_types::Text>>)]
    pub line_values: Vec<Cow<'a, str>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lyrics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Queryable, Selectable))]
pub struct Key<'a> {
    pub song_id: Uuid,
    pub description: Cow<'a, str>,
    pub external: bool,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lyrics, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[cfg_attr(test, derive(Queryable, Selectable))]
pub struct Upsert<'a> {
    #[diesel(embed)]
    pub key: Key<'a>,
    #[diesel(embed)]
    pub data: Data<'a>,
}
