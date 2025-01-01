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
    pub description: Option<Cow<'a, str>>,
    pub language: Cow<'a, str>,
    #[diesel(select_expression = sql("lyrics.durations durations"))]
    #[diesel(select_expression_type =
        SqlLiteral<sql_types::Nullable<sql_types::Array<sql_types::Integer>>>
    )]
    pub durations: Option<Vec<i32>>,
    #[diesel(select_expression = sql("lyrics.texts texts"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Array<sql_types::Text>>)]
    pub texts: Vec<Cow<'a, str>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lyrics, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Foreign {
    pub song_id: Uuid,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lyrics, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    #[diesel(embed)]
    pub foreign: Foreign,
    pub source: Option<Cow<'a, str>>,
    #[diesel(embed)]
    pub data: Data<'a>,
}

mod upsert {
    use diesel::{DecoratableTarget, ExpressionMethods};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Upsert, lyrics};
    use crate::Error;
    use crate::database::Database;

    impl crate::orm::upsert::Insert for Upsert<'_> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            if self.source.is_some() {
                diesel::insert_into(lyrics::table)
                    .values(self)
                    .on_conflict((lyrics::song_id, lyrics::source))
                    .do_update()
                    .set((&self.data, lyrics::scanned_at.eq(crate::time::now().await)))
                    .returning(lyrics::id)
                    .get_result(&mut database.get().await?)
                    .await
            } else {
                diesel::insert_into(lyrics::table)
                    .values(self)
                    .on_conflict((lyrics::song_id, lyrics::description))
                    .filter_target(lyrics::source.is_null())
                    .do_update()
                    .set((&self.data, lyrics::scanned_at.eq(crate::time::now().await)))
                    .returning(lyrics::id)
                    .get_result(&mut database.get().await?)
                    .await
            }
            .map_err(Error::from)
        }
    }
}

mod convert {
    use std::borrow::Cow;

    use crate::Error;
    use crate::file::lyric::{Lines, Lyrics};
    use crate::orm::lyrics;

    impl<'a> TryFrom<&'a Lyrics<'_>> for lyrics::Data<'a> {
        type Error = Error;

        fn try_from(value: &'a Lyrics<'_>) -> Result<Self, Error> {
            let (durations, texts) = match &value.lines {
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
            Ok(Self {
                description: value.description.as_deref().map(Cow::Borrowed),
                language: value.language.to_639_3().into(),
                durations,
                texts,
            })
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use std::borrow::Cow;

    use crate::file::lyric::Lyrics;
    use crate::orm::lyrics;

    impl From<lyrics::Data<'_>> for Lyrics<'static> {
        fn from(value: lyrics::Data<'_>) -> Self {
            Self {
                description: value.description.map(Cow::into_owned).map(Cow::Owned),
                language: value.language.parse().unwrap(),
                lines: if let Some(durations) = value.durations {
                    durations
                        .into_iter()
                        .zip(value.texts)
                        .map(|(duration, text)| (duration.try_into().unwrap(), text.into_owned()))
                        .collect()
                } else {
                    value.texts.into_iter().map(Cow::into_owned).collect()
                },
            }
        }
    }
}
