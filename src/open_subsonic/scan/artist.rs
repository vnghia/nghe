use std::borrow::Cow;

use anyhow::Result;
use diesel::{DecoratableTarget, ExpressionMethods};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use futures::TryStreamExt;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::config::ArtistIndexConfig;
use crate::models::*;
use crate::{DatabasePool, OSError};

pub async fn upsert_artists(
    pool: &DatabasePool,
    ignored_prefixes: &[String],
    artist_no_ids: &[artists::ArtistNoId],
) -> Result<Vec<Uuid>> {
    stream::iter(artist_no_ids)
        .then(|artist_no_id| async move {
            let index = compute_artist_index(ignored_prefixes, &artist_no_id.name)?;
            if artist_no_id.mbz_id.is_some() {
                diesel::insert_into(artists::table)
                    .values(artists::NewArtistWithIndex { new_artist: artist_no_id.into(), index })
                    .on_conflict(artists::mbz_id)
                    .do_update()
                    .set(artists::scanned_at.eq(time::OffsetDateTime::now_utc()))
                    .returning(artists::id)
                    .get_result::<Uuid>(&mut pool.get().await.map_err(anyhow::Error::from)?)
                    .await
            } else {
                diesel::insert_into(artists::table)
                    .values(artists::NewArtistWithIndex { new_artist: artist_no_id.into(), index })
                    .on_conflict(artists::name)
                    .filter_target(artists::mbz_id.is_null())
                    .do_update()
                    .set(artists::scanned_at.eq(time::OffsetDateTime::now_utc()))
                    .returning(artists::id)
                    .get_result::<Uuid>(&mut pool.get().await.map_err(anyhow::Error::from)?)
                    .await
            }
            .map_err(anyhow::Error::from)
        })
        .try_collect()
        .await
}

pub async fn insert_ignored_articles_config(
    pool: &DatabasePool,
    ignored_articles: &str,
) -> Result<()> {
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
    Ok(())
}

fn compute_artist_index<S: AsRef<str>>(
    ignored_prefixes: &[S],
    name: &str,
) -> Result<Cow<'static, str>> {
    let mut iter = ignored_prefixes.iter();
    let name = loop {
        match iter.next() {
            Some(ignored_prefix) => {
                if let Some(stripped) = name.strip_prefix(ignored_prefix.as_ref()) {
                    break stripped;
                }
            }
            None => break name,
        }
    };
    name.nfkd()
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!(OSError::InvalidParameter(
                "artist name is empty after stripping articles".into()
            ))
        })
        .map(|c| {
            if c.is_ascii_alphabetic() {
                c.to_ascii_uppercase().to_string().into()
            } else if c.is_numeric() {
                "#".into()
            } else if !c.is_alphabetic() {
                "*".into()
            } else {
                c.to_string().into()
            }
        })
}

#[cfg(test)]
mod tests {
    use diesel::QueryDsl;
    use fake::{Fake, Faker};
    use itertools::Itertools;

    use super::*;
    use crate::utils::test::TemporaryDb;

    async fn assert_artist_indexes<S: AsRef<str>>(
        pool: &DatabasePool,
        artist_no_ids: &[artists::ArtistNoId],
        ignored_prefixes: &[S],
    ) {
        assert_eq!(
            artist_no_ids
                .iter()
                .map(|artist_no_id| compute_artist_index(
                    ignored_prefixes,
                    artist_no_id.name.as_ref()
                )
                .unwrap())
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
    fn test_compute_artist_index_with_article() {
        assert_eq!(compute_artist_index(&["The ", "A "], "The test").unwrap(), "T");
    }

    #[test]
    fn test_compute_artist_index_number() {
        assert_eq!(compute_artist_index(&["The ", "A "], "The 1").unwrap(), "#");
    }

    #[test]
    fn test_compute_artist_index_no_article() {
        assert_eq!(compute_artist_index(&["The ", "A "], "test").unwrap(), "T");
    }

    #[test]
    fn test_compute_artist_index_non_ascii() {
        assert_eq!(compute_artist_index(&["The ", "A "], "狼").unwrap(), "狼");
    }

    #[test]
    fn test_compute_artist_index_decompose_ascii() {
        assert_eq!(compute_artist_index(&["The ", "A "], "é").unwrap(), "E");
    }

    #[test]
    fn test_compute_artist_index_decompose_non_ascii() {
        assert_eq!(compute_artist_index(&["The ", "A "], "ド").unwrap(), "ト");
    }

    #[test]
    fn test_compute_artist_index_compatibility() {
        assert_eq!(compute_artist_index(&["The ", "A "], "ａ").unwrap(), "A");
    }

    #[test]
    fn test_compute_artist_index_non_alphabetic() {
        assert_eq!(compute_artist_index(&["The ", "A "], "%").unwrap(), "*");
    }

    #[tokio::test]
    async fn test_build_artist_indexes() {
        let temp_db = TemporaryDb::new_from_env().await;
        let artist_no_ids = artists::ArtistNoId::fake_vec(10..=10);
        let artist_index_config = ArtistIndexConfig::default();

        upsert_artists(temp_db.pool(), &artist_index_config.ignored_prefixes, &artist_no_ids)
            .await
            .unwrap();
        assert_artist_indexes(
            temp_db.pool(),
            &artist_no_ids,
            &artist_index_config.ignored_prefixes,
        )
        .await;
    }

    #[tokio::test]
    async fn test_upsert_artist_mbz_id() {
        let temp_db = TemporaryDb::new_from_env().await;
        let mbz_id = Some(Faker.fake());
        let artist_no_id1 = artists::ArtistNoId { mbz_id, ..Faker.fake() };
        let artist_no_id2 = artists::ArtistNoId { mbz_id, ..Faker.fake() };
        let artist_index_config = ArtistIndexConfig::default();

        let artist_id1 =
            upsert_artists(temp_db.pool(), &artist_index_config.ignored_prefixes, &[artist_no_id1])
                .await
                .unwrap()
                .remove(0);
        let artist_id2 =
            upsert_artists(temp_db.pool(), &artist_index_config.ignored_prefixes, &[artist_no_id2])
                .await
                .unwrap()
                .remove(0);
        // Because they share the same mbz id
        assert_eq!(artist_id1, artist_id2);
    }

    #[tokio::test]
    async fn test_upsert_artist_name() {
        let temp_db = TemporaryDb::new_from_env().await;
        let artist_no_id1 = artists::ArtistNoId { name: "alias1".into(), mbz_id: None };
        let artist_no_id2 = artists::ArtistNoId { name: "alias1".into(), mbz_id: None };
        let artist_index_config = ArtistIndexConfig::default();

        let artist_id1 =
            upsert_artists(temp_db.pool(), &artist_index_config.ignored_prefixes, &[artist_no_id1])
                .await
                .unwrap()
                .remove(0);
        let artist_id2 =
            upsert_artists(temp_db.pool(), &artist_index_config.ignored_prefixes, &[artist_no_id2])
                .await
                .unwrap()
                .remove(0);
        // Because they share the same name and mbz id is null.
        assert_eq!(artist_id1, artist_id2);

        let artist_no_id1 =
            artists::ArtistNoId { name: "alias2".into(), mbz_id: Some(Faker.fake()) };
        let artist_no_id2 =
            artists::ArtistNoId { name: "alias2".into(), mbz_id: Some(Faker.fake()) };
        let artist_id1 =
            upsert_artists(temp_db.pool(), &artist_index_config.ignored_prefixes, &[artist_no_id1])
                .await
                .unwrap()
                .remove(0);
        let artist_id2 =
            upsert_artists(temp_db.pool(), &artist_index_config.ignored_prefixes, &[artist_no_id2])
                .await
                .unwrap()
                .remove(0);
        // Because they share the same name but their mbz ids are different.
        assert_ne!(artist_id1, artist_id2);
    }
}
