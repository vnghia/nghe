use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use futures::TryStreamExt;
use itertools::Itertools;
use std::borrow::Cow;
use uuid::Uuid;

pub async fn upsert_artists<S: AsRef<str>>(
    pool: &DatabasePool,
    names: &[S],
) -> OSResult<Vec<Uuid>> {
    Ok(diesel::insert_into(artists::table)
        .values(
            names
                .iter()
                .map(|name| artists::NewArtist {
                    name: std::borrow::Cow::Borrowed(name.as_ref()),
                })
                .collect_vec(),
        )
        .on_conflict(artists::name)
        .do_update()
        .set(artists::scanned_at.eq(time::OffsetDateTime::now_utc()))
        .returning(artists::id)
        .get_results(&mut pool.get().await?)
        .await?)
}

// TODO: better index building mechanism
fn build_artist_index<S: AsRef<str>>(ignored_prefixes: &[S], name: &str) -> Cow<'static, str> {
    for ignored_prefix in ignored_prefixes {
        if let Some(stripped) = name.strip_prefix(ignored_prefix.as_ref()) {
            if let Some(index_char) = stripped.chars().next() {
                return index_char_to_string(index_char);
            }
        }
    }

    if let Some(index_char) = name.chars().next() {
        index_char_to_string(index_char)
    } else {
        unreachable!("name can not be empty")
    }
}

pub async fn build_artist_indices<S: AsRef<str>>(
    pool: &DatabasePool,
    ignored_prefixes: &[S],
) -> OSResult<()> {
    stream::iter(
        artists::table
            .select((artists::id, artists::name))
            .filter(artists::index.eq("?"))
            .load::<(Uuid, String)>(&mut pool.get().await?)
            .await?,
    )
    .then(|(id, name)| async move {
        diesel::update(artists::table)
            .filter(artists::id.eq(id))
            .set(artists::index.eq(build_artist_index(ignored_prefixes, &name)))
            .execute(&mut pool.get().await?)
            .await?;
        OSResult::Ok(())
    })
    .try_collect()
    .await?;
    Ok(())
}

fn index_char_to_string(index_char: char) -> Cow<'static, str> {
    if index_char.is_ascii_alphabetic() {
        index_char.to_ascii_uppercase().to_string().into()
    } else if index_char.is_numeric() {
        Cow::Borrowed("#")
    } else {
        Cow::Borrowed("*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::db::TemporaryDatabase;

    use fake;

    #[test]
    fn test_index_char_to_string_numeric() {
        assert_eq!(index_char_to_string('1'), "#");
    }

    #[test]
    fn test_index_char_to_string_alphabetic_upper() {
        assert_eq!(index_char_to_string('A'), "A");
    }

    #[test]
    fn test_index_char_to_string_alphabetic_lower() {
        assert_eq!(index_char_to_string('a'), "A");
    }

    #[test]
    fn test_index_char_to_string_non_alphabetic() {
        assert_eq!(index_char_to_string('%'), "*");
    }

    #[test]
    fn test_index_char_to_string_non_ascii() {
        assert_eq!(index_char_to_string('é'), "*");
    }

    #[test]
    fn test_build_artist_index_with_article() {
        assert_eq!(build_artist_index(&["The ", "A "], "The test"), "T");
    }

    #[test]
    fn test_build_artist_index_no_article() {
        assert_eq!(build_artist_index(&["The ", "A "], "test"), "T");
    }

    #[tokio::test]
    async fn test_build_artist_indices() {
        let db = TemporaryDatabase::new_from_env().await;
        let artist_names = fake::vec![String; 10];
        let ignored_prefixes = ["The ", "A "];

        upsert_artists(db.get_pool(), &artist_names).await.unwrap();
        build_artist_indices(db.get_pool(), &ignored_prefixes)
            .await
            .unwrap();

        assert_eq!(
            artist_names
                .iter()
                .map(|name| build_artist_index(&ignored_prefixes, name))
                .sorted()
                .collect_vec(),
            artists::table
                .select(artists::index)
                .load::<String>(&mut db.get_pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec()
        );
    }
}
