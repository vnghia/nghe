use std::borrow::Cow;

use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use o2o::o2o;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::genres;
use crate::Error;

#[derive(Debug, o2o)]
#[ref_into(genres::Upsert<'a>)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Genre<'a> {
    #[ref_into(Cow::Borrowed(~.as_ref()))]
    #[cfg_attr(test, dummy(expr = "Faker.fake::<String>().into()"))]
    pub value: Cow<'a, str>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Genres<'a> {
    #[cfg_attr(test, dummy(expr = "fake::vec![Genre<'static>; 0..=2]"))]
    pub value: Vec<Genre<'a>>,
}

impl<'a> From<&'a str> for Genre<'a> {
    fn from(genre: &'a str) -> Self {
        Self { value: genre.into() }
    }
}

impl<'a> FromIterator<&'a str> for Genres<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self { value: iter.into_iter().map(Genre::from).collect() }
    }
}

impl<'a, 'b> From<&'a Genres<'b>> for Vec<genres::Upsert<'b>>
where
    'a: 'b,
{
    fn from(value: &'a Genres<'b>) -> Self {
        value.value.iter().map(<&Genre>::into).collect()
    }
}

impl<'a> Genres<'a> {
    pub async fn upsert(&self, database: &Database) -> Result<Vec<Uuid>, Error> {
        diesel::insert_into(genres::table)
            .values::<Vec<genres::Upsert<'_>>>(self.into())
            .on_conflict(genres::value)
            .do_update()
            .set(genres::upserted_at.eq(time::OffsetDateTime::now_utc()))
            .returning(genres::id)
            .get_results(&mut database.get().await?)
            .await
            .map_err(Error::from)
    }
}
