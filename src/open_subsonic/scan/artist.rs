use std::borrow::Cow;

use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, PgExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use futures::TryStreamExt;
use itertools::Itertools;
use uuid::Uuid;

use crate::config::ArtistIndexConfig;
use crate::models::*;
use crate::DatabasePool;

pub async fn upsert_artists<S: AsRef<str>>(pool: &DatabasePool, names: &[S]) -> Result<Vec<Uuid>> {
    diesel::insert_into(artists::table)
        .values(
            names
                .iter()
                .map(|name| artists::NewArtist { name: name.as_ref().into() })
                .collect_vec(),
        )
        .on_conflict(artists::name)
        .do_update()
        .set(artists::scanned_at.eq(time::OffsetDateTime::now_utc()))
        .returning(artists::id)
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

// TODO: better index building mechanism
fn build_artist_index<S: AsRef<str>>(ignored_prefixes: &[S], name: &str) -> Cow<'static, str> {
    for ignored_prefix in ignored_prefixes {
        if let Some(stripped) = name.strip_prefix(ignored_prefix.as_ref())
            && let Some(index_char) = stripped.chars().next()
        {
            return index_char_to_string(index_char);
        }
    }

    if let Some(index_char) = name.chars().next() {
        index_char_to_string(index_char)
    } else {
        unreachable!("name can not be empty")
    }
}

pub async fn build_artist_indices(
    pool: &DatabasePool,
    ArtistIndexConfig { ignored_articles, ignored_prefixes }: &ArtistIndexConfig,
) -> Result<()> {
    let artist_ids_names = {
        let mut artist_query = artists::table.select((artists::id, artists::name)).into_boxed();

        let need_full_rebuild = configs::table
            .select(configs::text.is_distinct_from(ignored_articles))
            .filter(configs::key.eq(ArtistIndexConfig::IGNORED_ARTICLES_CONFIG_KEY))
            .first::<bool>(&mut pool.get().await?)
            .await
            .optional()?
            .unwrap_or(true); // None if the key hasn't been added to the table yet.
        if !need_full_rebuild {
            artist_query = artist_query.filter(artists::index.eq("?"));
        }

        artist_query.load::<(Uuid, String)>(&mut pool.get().await?).await?
    };

    if !artist_ids_names.is_empty() {
        stream::iter(artist_ids_names)
            .then(|(id, name)| async move {
                diesel::update(artists::table)
                    .filter(artists::id.eq(id))
                    .set(artists::index.eq(build_artist_index(ignored_prefixes, &name)))
                    .execute(&mut pool.get().await?)
                    .await?;
                Result::<_, anyhow::Error>::Ok(())
            })
            .try_collect()
            .await?;
        diesel::insert_into(configs::table)
            .values(&configs::NewTextConfig {
                key: ArtistIndexConfig::IGNORED_ARTICLES_CONFIG_KEY.into(),
                text: ignored_articles.into(),
            })
            .on_conflict(configs::key)
            .do_update()
            .set(configs::text.eq(ignored_articles))
            .execute(&mut pool.get().await?)
            .await?;
    }

    Ok(())
}

fn index_char_to_string(index_char: char) -> Cow<'static, str> {
    if index_char.is_ascii_alphabetic() {
        index_char.to_ascii_uppercase().to_string().into()
    } else if index_char.is_numeric() {
        "#".into()
    } else {
        "*".into()
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;

    use super::*;
    use crate::utils::test::TemporaryDb;

    async fn assert_artist_indices<SN: AsRef<str>, SP: AsRef<str>>(
        pool: &DatabasePool,
        artist_names: &[SN],
        ignored_prefixes: &[SP],
    ) {
        assert_eq!(
            artist_names
                .iter()
                .map(|name| build_artist_index(ignored_prefixes, name.as_ref()))
                .sorted()
                .collect_vec(),
            artists::table
                .select(artists::index)
                .load::<String>(&mut pool.get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec()
        );
    }

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
        let temp_db = TemporaryDb::new_from_env().await;
        let artist_names = fake::vec![String; 10];
        let artist_index_config = ArtistIndexConfig::new("The A".to_owned());

        upsert_artists(temp_db.pool(), &artist_names).await.unwrap();
        build_artist_indices(temp_db.pool(), &artist_index_config).await.unwrap();

        assert_artist_indices(temp_db.pool(), &artist_names, &artist_index_config.ignored_prefixes)
            .await;
    }

    #[tokio::test]
    async fn test_build_artist_indices_full_rebuild() {
        let temp_db = TemporaryDb::new_from_env().await;
        let artist_names = fake::vec![String; 10];
        let artist_index_config = ArtistIndexConfig::new("The A".to_owned());

        upsert_artists(temp_db.pool(), &artist_names).await.unwrap();
        build_artist_indices(temp_db.pool(), &artist_index_config).await.unwrap();
        assert_artist_indices(temp_db.pool(), &artist_names, &artist_index_config.ignored_prefixes)
            .await;

        let artist_index_config = ArtistIndexConfig::new("Le La".to_owned());
        build_artist_indices(temp_db.pool(), &artist_index_config).await.unwrap();
        assert_artist_indices(temp_db.pool(), &artist_names, &artist_index_config.ignored_prefixes)
            .await;
    }

    #[tokio::test]
    async fn test_build_artist_indices_partial_rebuild() {
        let temp_db = TemporaryDb::new_from_env().await;
        let artist_names = fake::vec![String; 10];
        let artist_index_config = ArtistIndexConfig::new("The A".to_owned());

        let artist_ids = upsert_artists(temp_db.pool(), &artist_names).await.unwrap();
        build_artist_indices(temp_db.pool(), &artist_index_config).await.unwrap();
        assert_artist_indices(temp_db.pool(), &artist_names, &artist_index_config.ignored_prefixes)
            .await;

        let artist_update_index_ids =
            artist_ids.choose_multiple(&mut rand::thread_rng(), 5).cloned().sorted().collect_vec();
        let update_count = diesel::update(artists::table)
            .filter(artists::id.eq_any(&artist_update_index_ids))
            .set(artists::index.eq("?"))
            .execute(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(update_count, artist_update_index_ids.len());

        let current_time = time::OffsetDateTime::now_utc();
        build_artist_indices(temp_db.pool(), &artist_index_config).await.unwrap();
        assert_artist_indices(temp_db.pool(), &artist_names, &artist_index_config.ignored_prefixes)
            .await;

        assert_eq!(
            artist_update_index_ids,
            artists::table
                .select(artists::id)
                .filter(artists::updated_at.ge(current_time))
                .get_results::<Uuid>(&mut temp_db.pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec()
        );
    }
}
