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
pub struct Lyrics<'a> {
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
    pub description: Cow<'a, str>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lyrics, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Foreign {
    pub song_id: Uuid,
    pub external: bool,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lyrics, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[cfg_attr(test, derive(Queryable, Selectable))]
pub struct Data<'a> {
    #[diesel(embed)]
    pub key: Key<'a>,
    #[diesel(embed)]
    pub lyrics: Lyrics<'a>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lyrics, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    #[diesel(embed)]
    pub foreign: Foreign,
    #[diesel(embed)]
    pub data: Data<'a>,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;

    use super::{Upsert, lyrics};
    use crate::Error;
    use crate::database::Database;

    impl Upsert<'_> {
        pub async fn upsert(&self, database: &Database) -> Result<(), Error> {
            diesel::insert_into(lyrics::table)
                .values(self)
                .on_conflict((lyrics::song_id, lyrics::external, lyrics::description))
                .do_update()
                .set((&self.data.lyrics, lyrics::scanned_at.eq(crate::time::now().await)))
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}

mod convert {
    use crate::Error;
    use crate::file::lyrics::{Lines, Lyrics};
    use crate::orm::lyrics;

    impl<'a> TryFrom<&'a Lyrics<'_>> for lyrics::Data<'a> {
        type Error = Error;

        fn try_from(value: &'a Lyrics<'_>) -> Result<Self, Error> {
            let (line_starts, line_values) = match &value.lines {
                Lines::Unsync(lines) => {
                    (None, lines.iter().map(|line| line.as_str().into()).collect())
                }
                Lines::Sync(lines) => {
                    let (durations, texts) = lines
                        .iter()
                        .map(|(duration, text)| {
                            Ok::<_, Error>(((*duration).try_into()?, text.as_str().into()))
                        })
                        .try_collect::<Vec<(i32, _)>>()?
                        .into_iter()
                        .unzip();
                    (Some(durations), texts)
                }
            };
            let lyrics = lyrics::Lyrics {
                language: value.language.to_639_3().into(),
                line_starts,
                line_values,
            };
            let key = lyrics::Key {
                description: value
                    .description
                    .as_ref()
                    .map_or_else(|| "".into(), |description| description.as_str().into()),
            };
            Ok(Self { key, lyrics })
        }
    }
}
