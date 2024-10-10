use std::borrow::Cow;

use diesel::dsl::{exists, not};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use o2o::o2o;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{genres, songs_genres};
use crate::Error;

#[derive(Debug, o2o)]
#[ref_into(genres::Data<'a>)]
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

impl<'a, S: Into<Cow<'a, str>>> From<S> for Genre<'a> {
    fn from(genre: S) -> Self {
        Self { value: genre.into() }
    }
}

impl<'a, S: Into<Cow<'a, str>>> FromIterator<S> for Genres<'a> {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self { value: iter.into_iter().map(Genre::from).collect() }
    }
}

impl<'a, 'b> From<&'a Genres<'b>> for Vec<genres::Data<'b>>
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
            .values::<Vec<genres::Data<'_>>>(self.into())
            .on_conflict(genres::value)
            .do_update()
            .set(genres::upserted_at.eq(time::OffsetDateTime::now_utc()))
            .returning(genres::id)
            .get_results(&mut database.get().await?)
            .await
            .map_err(Error::from)
    }

    pub async fn upsert_song(
        database: &Database,
        song_id: Uuid,
        genre_ids: &[Uuid],
    ) -> Result<(), Error> {
        diesel::insert_into(songs_genres::table)
            .values::<Vec<_>>(
                genre_ids
                    .iter()
                    .copied()
                    .map(|genre_id| songs_genres::Data { song_id, genre_id })
                    .collect(),
            )
            .on_conflict((songs_genres::song_id, songs_genres::genre_id))
            .do_update()
            .set(songs_genres::upserted_at.eq(time::OffsetDateTime::now_utc()))
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }

    pub async fn cleanup_one(
        database: &Database,
        started_at: time::OffsetDateTime,
        song_id: Uuid,
    ) -> Result<(), Error> {
        // Delete all genres of a song which haven't been refreshed since timestamp.
        diesel::delete(songs_genres::table)
            .filter(songs_genres::song_id.eq(song_id))
            .filter(songs_genres::upserted_at.lt(started_at))
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }

    pub async fn cleanup(database: &Database) -> Result<(), Error> {
        // Delete all genres which do not have any song associated.
        let alias_genres = diesel::alias!(genres as alias_genres);
        diesel::delete(genres::table)
            .filter(
                genres::id.eq_any(
                    alias_genres
                        .filter(not(exists(
                            songs_genres::table
                                .filter(songs_genres::genre_id.eq(alias_genres.field(genres::id))),
                        )))
                        .select(alias_genres.field(genres::id)),
                ),
            )
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;

    use super::*;
    use crate::orm::songs;
    use crate::test::Mock;

    impl Genres<'static> {
        pub async fn query(mock: &Mock, song_id: Uuid) -> Self {
            songs_genres::table
                .inner_join(songs::table)
                .inner_join(genres::table)
                .filter(songs::id.eq(song_id))
                .select(genres::value)
                .get_results::<String>(&mut mock.get().await)
                .await
                .unwrap()
                .into_iter()
                .collect()
        }
    }
}
